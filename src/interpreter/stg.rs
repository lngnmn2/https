//! # STG Machine Implementation (Spineless Tagless G-Machine)
//! 
//! Pure functional implementation using recursion and persistent data.

use crate::domain::{HttpError, Response, Host, Port, Status, Body, SecurityLevel};
use crate::interpreter::{socket, tls, protocol};
use openssl::ssl::SslStream;
use std::net::TcpStream;
use std::rc::Rc;
use std::ops::Deref;

/// Pointer into the persistent graph.
pub type Addr = usize;

// --- PERSISTENT DATA STRUCTURES ---

/// Immutable Linked List for the STG Stack.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum List<T> { 
    /// End of list.
    Nil, 
    /// A shared value and the tail.
    Cons(Rc<T>, Rc<List<T>>) 
}

impl<T> List<T> {
    /// Pushes a value onto the list.
    pub fn push(self: Rc<Self>, val: T) -> Rc<Self> { Rc::new(List::Cons(Rc::new(val), self)) }
    /// Pops a value from the list (Returns Rc-wrapped tail).
    pub fn pop_rc(self: Rc<Self>) -> Option<(Rc<T>, Rc<List<T>>)> {
        match self.deref() {
            List::Nil => None,
            List::Cons(h, t) => Some((h.clone(), t.clone())),
        }
    }
}

// --- INITIAL ALGEBRA (STG CORE SYNTAX) ---

/// Atomic values (WHNF or Pointers).
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Atom<A> { 
    /// A literal value.
    Lit(Rc<A>), 
    /// A heap address.
    Var(Addr) 
}

impl<A> Clone for Atom<A> {
    fn clone(&self) -> Self {
        match self {
            Atom::Lit(v) => Atom::Lit(v.clone()),
            Atom::Var(a) => Atom::Var(*a),
        }
    }
}

/// The Expression AST (Core Language).
pub enum Expr<A> {
    /// Evaluate an atom.
    Pure(Atom<A>), 
    /// Enter a heap address.
    App(Addr),
    /// Monadic Bind (Strict evaluation).
    Case(Box<Expr<A>>, Box<dyn FnOnce(Response) -> Expr<A> + 'static>),
    /// Lazy allocation.
    Let(Box<Closure<A>>, Box<dyn Fn(Addr) -> Expr<A> + 'static>),
    /// TCP connection primitive.
    OpConnect(Host, Port), 
    /// TLS handshake primitive.
    OpHandshake(Host), 
    /// Write primitive.
    OpWrite(Rc<[u8]>), 
    /// Read primitive with security level.
    OpRead(SecurityLevel),
}

/// A Heap Closure (Entry Code + Payload).
pub struct Closure<A> { 
    /// True if the result should be memoized.
    pub is_updatable: bool, 
    /// The code to execute.
    pub expr: Box<Expr<A>> 
}

// --- RUNTIME STATE ---

/// A Node in the persistent graph (Heap).
pub enum Node<A> { 
    /// A suspended computation.
    Closure(Box<Closure<A>>), 
    /// A memoized value.
    Whnf(Rc<A>), 
    /// A pointer to another node.
    Indirection(Addr), 
    /// Loop sentinel.
    Blackhole 
}

/// STG Stack Frames (Continuations).
pub enum Frame<A> { 
    /// Update the heap with the result.
    Update(Addr), 
    /// Return to a Case continuation.
    ReturnTo(Box<dyn FnOnce(Response) -> Expr<A> + 'static>) 
}

/// Control signal for the recursive evaluation loop.
pub enum Control<A> { 
    /// Continue evaluation with the next expression.
    Continue(Expr<A>), 
    /// Halt evaluation with a final value.
    Halt(Rc<A>) 
}

/// The intermediate state of a network connection.
#[derive(Debug)]
pub enum Connection { 
    /// Unconnected.
    Empty, 
    /// TCP connected.
    Tcp(TcpStream), 
    /// TLS established.
    Tls(SslStream<TcpStream>) 
}

/// The Persistent STG Machine State.
pub struct StgMachine<A> {
    /// The shared heap.
    pub heap: Rc<[Node<A>]>,
    /// The persistent stack.
    pub stack: Rc<List<Frame<A>>>,
    /// The network connection state.
    pub conn: Connection,
}

impl<A: 'static> StgMachine<A> {
    /// Creates a new, empty machine.
    pub fn new() -> Self {
        Self { heap: Rc::from([]), stack: Rc::new(List::Nil), conn: Connection::Empty }
    }

    /// Pure Recursive Evaluation.
    pub fn evaluate(self, expr: Expr<A>) -> Result<Rc<A>, HttpError> {
        let (next_machine, control) = self.step(expr)?;
        match control {
            Control::Continue(e) => next_machine.evaluate(e),
            Control::Halt(v) => Ok(v),
        }
    }

    fn step(self, expr: Expr<A>) -> Result<(Self, Control<A>), HttpError> {
        match expr {
            Expr::Let(c, f) => {
                let addr = self.heap.len();
                let next_heap = self.heap.iter().cloned().chain(std::iter::once(Node::Closure(c))).collect::<Rc<[_]>>();
                Ok((StgMachine { heap: next_heap, ..self }, Control::Continue(f(addr))))
            }
            Expr::Case(t, k) => {
                let next_stack = self.stack.push(Frame::ReturnTo(k));
                Ok((StgMachine { stack: next_stack, ..self }, Control::Continue(*t)))
            }
            Expr::App(a) => {
                let (m, e) = self.enter_address(a)?;
                Ok((m, Control::Continue(e)))
            }
            Expr::Pure(atom) => {
                let (m, v) = self.unwrap_atom(atom)?;
                m.rule_ret(v)
            }
            p => {
                let (m, e) = self.rule_prim(p)?;
                Ok((m, Control::Continue(e)))
            }
        }
    }

    fn rule_ret(self, v: Rc<A>) -> Result<(Self, Control<A>), HttpError> {
        match self.stack.pop_rc() {
            None => Ok((StgMachine { stack: Rc::new(List::Nil), ..self }, Control::Halt(v))),
            Some((frame, next_stack)) => match frame.deref() {
                Frame::Update(upd) => {
                    let next_heap = self.heap.iter().enumerate().map(|(i, n)| if i == *upd { Node::Whnf(v.clone()) } else { n.clone() }).collect::<Rc<[_]>>();
                    Ok((StgMachine { heap: next_heap, stack: next_stack, ..self }, Control::Continue(Expr::Pure(Atom::Lit(v)))))
                }
                _ => Err(HttpError::RuntimeError(Rc::from("Return Type Mismatch"))),
            }
        }
    }

    fn rule_prim(self, p: Expr<A>) -> Result<(Self, Expr<A>), HttpError> {
        match p {
            Expr::OpConnect(h, p) => {
                let conn = handle_connect(&*h, p.code())?;
                StgMachine { conn, ..self }.yield_r(dummy())
            }
            Expr::OpHandshake(h) => {
                let conn = handle_handshake(self.conn, &*h)?;
                StgMachine { conn, ..self }.yield_r(dummy())
            }
            Expr::OpWrite(d) => {
                let conn = handle_write(self.conn, &*d)?;
                StgMachine { conn, ..self }.yield_r(dummy())
            }
            Expr::OpRead(level) => match self.conn {
                Connection::Tls(s) => {
                    let (next_s, res) = protocol::read_response_pure(s, level)?;
                    StgMachine { conn: Connection::Tls(next_s), ..self }.yield_r(res)
                }
                _ => Err(HttpError::RuntimeError(Rc::from("Invalid State for Read"))),
            },
            _ => Err(HttpError::RuntimeError(Rc::from("Invalid Primitive"))),
        }
    }

    fn yield_r(self, r: Response) -> Result<(Self, Expr<A>), HttpError> {
        match self.stack.pop_rc() {
            Some((frame, next_stack)) => match Rc::try_unwrap(frame).ok() {
                Some(Frame::ReturnTo(k)) => Ok((StgMachine { stack: next_stack, ..self }, k(r))),
                _ => Err(HttpError::RuntimeError(Rc::from("Frame Unwrapping Failed"))),
            },
            _ => Err(HttpError::RuntimeError(Rc::from("Stack Underflow"))),
        }
    }

    fn unwrap_atom(self, a: Atom<A>) -> Result<(Self, Rc<A>), HttpError> {
        match a {
            Atom::Lit(v) => Ok((self, v)),
            Atom::Var(addr) => {
                let (m, e) = self.enter_address(addr)?;
                match e {
                    Expr::Pure(Atom::Lit(v)) => Ok((m, v)),
                    _ => Err(HttpError::RuntimeError(Rc::from("Atom Resolution Failed"))),
                }
            }
        }
    }

    fn enter_address(self, addr: Addr) -> Result<(Self, Expr<A>), HttpError> {
        let node = self.heap.get(addr).cloned().ok_or(HttpError::RuntimeError(Rc::from("Invalid Addr")))?;
        match node {
            Node::Closure(c) => {
                let next_heap = self.heap.iter().enumerate().map(|(i, n)| if i == addr { Node::Blackhole } else { n.clone() }).collect::<Rc<[_]>>();
                let next_stack = if c.is_updatable { self.stack.push(Frame::Update(addr)) } else { self.stack };
                Ok((StgMachine { heap: next_heap, stack: next_stack, ..self }, *c.expr))
            }
            Node::Indirection(a) => self.enter_address(a),
            Node::Whnf(v) => Ok((self, Expr::Pure(Atom::Lit(v)))),
            Node::Blackhole => Err(HttpError::RuntimeError(Rc::from("Blackhole"))),
        }
    }
}

fn dummy() -> Response { Response::new(Status::Ok, Rc::from([]), Body::default()) }
fn handle_connect(h: &str, p: u16) -> Result<Connection, HttpError> {
    let s = socket::connect_tcp(h, p)?;
    let t = std::time::Duration::from_secs(10);
    let _ = s.set_read_timeout(Some(t))?;
    let _ = s.set_write_timeout(Some(t))?;
    Ok(Connection::Tcp(s))
}
fn handle_handshake(st: Connection, h: &str) -> Result<Connection, HttpError> {
    match st { Connection::Tcp(s) => Ok(Connection::Tls(tls::connect_tls(h, s)?)), _ => Err(HttpError::RuntimeError(Rc::from("Handshake Invariant"))) }
}
fn handle_write(st: Connection, d: &[u8]) -> Result<Connection, HttpError> {
    match st { 
        Connection::Tls(s) => {
            let next_s = tls::write_all_pure(s, d)?;
            Ok(Connection::Tls(next_s))
        } 
        _ => Err(HttpError::RuntimeError(Rc::from("Write Invariant"))) 
    }
}

impl<A> Clone for Node<A> { fn clone(&self) -> Self { match self { Node::Closure(c) => Node::Closure(Box::new(Closure { is_updatable: c.is_updatable, expr: c.expr.clone() })), Node::Whnf(v) => Node::Whnf(v.clone()), Node::Indirection(a) => Node::Indirection(*a), Node::Blackhole => Node::Blackhole } } }
impl<A> Clone for Expr<A> { 
    fn clone(&self) -> Self { 
        match self { 
            Expr::Pure(a) => Expr::Pure(a.clone()), 
            Expr::App(a) => Expr::App(*a), 
            _ => panic!("Unclonable Expr") 
        } 
    } 
}

// --- MANUAL DEBUG IMPLEMENTATIONS ---
impl<A> std::fmt::Debug for Expr<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Expr") } }
impl<A> std::fmt::Debug for Closure<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Closure") } }
impl<A> std::fmt::Debug for Node<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Node") } }
impl<A> std::fmt::Debug for Frame<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Frame") } }
impl<A> std::fmt::Debug for Control<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "Control") } }
impl<A> std::fmt::Debug for StgMachine<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("StgMachine").field("heap_len", &self.heap.len()).field("conn", &self.conn).finish() } }
impl<A: 'static> Default for StgMachine<A> { fn default() -> Self { Self::new() } }
