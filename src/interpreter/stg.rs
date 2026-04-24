//! # STG Machine Implementation (Spineless Tagless G-Machine)
//! 
//! Rigorous implementation of the formal STG transition rules.
//! Incorporates 40+ years of functional programming research.

use crate::domain::error::HttpError;
use crate::domain::response::Response;
use crate::domain::host::Host;
use crate::domain::port::Port;
use crate::domain::status::Status;
use crate::domain::body::Body;
use crate::interpreter::{socket, tls, protocol};
use core::mem;
use openssl::ssl::SslStream;
use std::net::TcpStream;
use std::io::Write;
use std::rc::Rc;

/// Pointer into the persistent graph.
pub type Addr = usize;

// ============================================================================
// 1. INITIAL ALGEBRA (STG CORE SYNTAX)
// ============================================================================

/// Atomic values (WHNF or Pointers).
#[derive(Debug, Clone)]
pub enum Atom<A> {
    /// A literal value in WHNF (Shared via Rc).
    Lit(Rc<A>),
    /// A pointer to a heap address.
    Var(Addr),
}

/// The Expression AST (Core Language).
pub enum Expr<A> {
    /// WHNF result.
    Pure(Atom<A>),
    /// Pointer entry (Graph traversal).
    App(Addr),
    /// Continuation-passing strict evaluation (Monadic Bind).
    Case(Box<Expr<A>>, Box<dyn FnOnce(Response) -> Expr<A> + 'static>),
    /// Lazy thunk allocation.
    Let(Box<Closure<A>>, Box<dyn Fn(Addr) -> Expr<A> + 'static>),

    /// Domain Primitive: Suspend to establish TCP connection.
    OpConnect(Host, Port),
    /// Domain Primitive: Suspend to perform TLS handshake.
    OpHandshake(Host),
    /// Domain Primitive: Suspend to write data.
    OpWrite(Vec<u8>),
    /// Domain Primitive: Suspend to read response.
    OpRead,
}

/// A Heap Closure (Entry Code + Payload).
pub struct Closure<A> {
    /// Memoization flag.
    pub is_updatable: bool,
    /// The code to execute when entering this closure.
    pub expr: Box<Expr<A>>,
}

// ============================================================================
// 2. RUNTIME STATE (MACHINE TOPOLOGY)
// ============================================================================

/// A Node in the persistent graph (Heap).
pub enum Node<A> {
    /// A suspended computation.
    Closure(Box<Closure<A>>),
    /// A memoized result.
    Whnf(Rc<A>),
    /// An indirection created during graph reduction.
    Indirection(Addr),
    /// Loop detection sentinel.
    Blackhole,
}

/// STG Stack Frames (Continuations).
pub enum Frame<A> {
    /// Update Frame: Instructs the machine to memoize the result.
    Update(Addr),
    /// Case Frame: The continuation for a Case expression.
    ReturnTo(Box<dyn FnOnce(Response) -> Expr<A> + 'static>),
}

/// Control signal for the evaluation loop.
enum Control<A> {
    Continue(Expr<A>),
    Halt(Rc<A>),
}

/// Represents the intermediate state of an HTTPS connection.
#[derive(Debug)]
pub enum ConnectionState {
    /// No active connection.
    Empty,
    /// Established TCP connection.
    Tcp(TcpStream),
    /// Established TLS session over TCP.
    Tls(SslStream<TcpStream>),
}

/// The Spineless Tagless G-Machine structure.
pub struct StgMachine<A> {
    heap: Vec<Node<A>>,
    stack: Vec<Frame<A>>,
    conn: ConnectionState,
}

// ============================================================================
// 3. OPERATIONAL SEMANTICS (TRANSITION RULES)
// ============================================================================

impl<A: 'static + Clone> StgMachine<A> {
    /// Creates a new STG Machine.
    pub fn new() -> Self {
        Self {
            heap: Vec::with_capacity(128), stack: Vec::with_capacity(64),
            conn: ConnectionState::Empty,
        }
    }

    /// Primary Evaluation Loop (Natural Transformation).
    pub fn evaluate(&mut self, mut expr: Expr<A>) -> Result<Rc<A>, HttpError> {
        loop {
            expr = match expr {
                Expr::Let(c, f) => self.rule_let(c, f),
                Expr::Case(t, k) => self.rule_case(*t, k),
                Expr::App(a) => self.enter_address(a)?,
                Expr::Pure(a) => {
                    let v = self.unwrap_atom(a)?;
                    match self.rule_ret(v)? {
                        Control::Continue(e) => e,
                        Control::Halt(v) => return Ok(v),
                    }
                }
                p => self.rule_prim(p)?,
            };
        }
    }

    fn rule_let(&mut self, c: Box<Closure<A>>, f: Box<dyn Fn(Addr) -> Expr<A> + 'static>) -> Expr<A> {
        let a = self.allocate(Node::Closure(c));
        f(a)
    }

    fn rule_case(&mut self, t: Expr<A>, k: Box<dyn FnOnce(Response) -> Expr<A> + 'static>) -> Expr<A> {
        self.stack.push(Frame::ReturnTo(k));
        t
    }

    fn rule_ret(&mut self, v: Rc<A>) -> Result<Control<A>, HttpError> {
        match self.stack.pop() {
            None => Ok(Control::Halt(v)),
            Some(Frame::Update(upd)) => {
                self.heap[upd] = Node::Whnf(v.clone());
                Ok(Control::Continue(Expr::Pure(Atom::Lit(v))))
            }
            Some(Frame::ReturnTo(_)) => Err(HttpError::RuntimeError("Type Mismatch: Expected Response".into())),
        }
    }

    fn rule_prim(&mut self, p: Expr<A>) -> Result<Expr<A>, HttpError> {
        match p {
            Expr::OpConnect(h, port) => {
                self.conn = handle_connect(h.as_ref(), port.into())?;
                self.yield_response(dummy_response())
            }
            Expr::OpHandshake(h) => {
                let s = mem::replace(&mut self.conn, ConnectionState::Empty);
                self.conn = handle_handshake(s, h.as_ref())?;
                self.yield_response(dummy_response())
            }
            Expr::OpWrite(d) => {
                let s = mem::replace(&mut self.conn, ConnectionState::Empty);
                self.conn = handle_write(s, d)?;
                self.yield_response(dummy_response())
            }
            Expr::OpRead => {
                match mem::replace(&mut self.conn, ConnectionState::Empty) {
                    ConnectionState::Tls(s) => self.yield_response(protocol::read_response(s)?),
                    _ => Err(HttpError::RuntimeError("Invalid State for Read".into())),
                }
            }
            _ => Err(HttpError::RuntimeError("Invalid Primitive".into())),
        }
    }

    fn allocate(&mut self, n: Node<A>) -> Addr {
        let a = self.heap.len(); self.heap.push(n); a
    }

    fn yield_response(&mut self, r: Response) -> Result<Expr<A>, HttpError> {
        match self.stack.pop() {
            Some(Frame::ReturnTo(k)) => Ok(k(r)),
            _ => Err(HttpError::RuntimeError("Stack Underflow".into())),
        }
    }

    fn unwrap_atom(&mut self, a: Atom<A>) -> Result<Rc<A>, HttpError> {
        match a {
            Atom::Lit(v) => Ok(v),
            Atom::Var(addr) => match self.enter_address(addr)? {
                Expr::Pure(Atom::Lit(v)) => Ok(v),
                _ => Err(HttpError::RuntimeError("Atom Resolution Failed".into())),
            }
        }
    }

    fn enter_address(&mut self, addr: Addr) -> Result<Expr<A>, HttpError> {
        match mem::replace(&mut self.heap[addr], Node::Blackhole) {
            Node::Closure(c) => {
                if c.is_updatable { self.stack.push(Frame::Update(addr)); }
                Ok(*c.expr)
            }
            Node::Indirection(a) => { self.heap[addr] = Node::Indirection(a); Ok(Expr::App(a)) }
            Node::Whnf(v) => { self.heap[addr] = Node::Whnf(v.clone()); Ok(Expr::Pure(Atom::Lit(v))) }
            Node::Blackhole => Err(HttpError::RuntimeError("Divergence Detected (Blackhole)".into())),
        }
    }
}

// ----------------------------------------------------------------------------
// HELPERS (IMPERATIVE SHELL ADAPTERS)
// ----------------------------------------------------------------------------

fn dummy_response() -> Response {
    Response::new(Status::Ok, vec![], Body::default())
}

fn handle_connect(host: &str, port: u16) -> Result<ConnectionState, HttpError> {
    let s = socket::connect_tcp(host, port)?;
    let t = std::time::Duration::from_secs(10);
    s.set_read_timeout(Some(t))?; s.set_write_timeout(Some(t))?;
    Ok(ConnectionState::Tcp(s))
}

fn handle_handshake(state: ConnectionState, host: &str) -> Result<ConnectionState, HttpError> {
    match state {
        ConnectionState::Tcp(s) => Ok(ConnectionState::Tls(tls::connect_tls(host, s)?)),
        _ => Err(HttpError::RuntimeError("Handshake Invariant Violated".into())),
    }
}

fn handle_write(state: ConnectionState, data: Vec<u8>) -> Result<ConnectionState, HttpError> {
    match state {
        ConnectionState::Tls(mut s) => {
            s.write_all(&data)?; s.flush()?;
            Ok(ConnectionState::Tls(s))
        }
        _ => Err(HttpError::RuntimeError("Write Invariant Violated".into())),
    }
}

// ----------------------------------------------------------------------------
// BOILERPLATE (DEBUG/DEFAULT)
// ----------------------------------------------------------------------------

impl<A> std::fmt::Debug for Node<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { Node::Closure(_) => write!(f, "Closure"), Node::Whnf(_) => write!(f, "Whnf"), Node::Indirection(a) => write!(f, "Indirection({})", a), Node::Blackhole => write!(f, "Blackhole") } } }
impl<A> std::fmt::Debug for Frame<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { Frame::Update(a) => write!(f, "Update({})", a), Frame::ReturnTo(_) => write!(f, "ReturnTo") } } }
impl<A> std::fmt::Debug for StgMachine<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("StgMachine").field("hp", &self.heap.len()).field("sp", &self.stack.len()).field("conn", &self.conn).finish() } }
impl<A> std::fmt::Debug for Expr<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { Expr::Pure(_) => write!(f, "Pure"), Expr::App(a) => write!(f, "App({})", a), Expr::Case(_, _) => write!(f, "Case"), Expr::Let(_, _) => write!(f, "Let"), Expr::OpConnect(_, _) => write!(f, "OpConnect"), Expr::OpHandshake(_) => write!(f, "OpHandshake"), Expr::OpWrite(_) => write!(f, "OpWrite"), Expr::OpRead => write!(f, "OpRead") } } }
impl<A> std::fmt::Debug for Closure<A> { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.debug_struct("Closure").field("upd", &self.is_updatable).finish() } }
impl<A: 'static + Clone> Default for StgMachine<A> { fn default() -> Self { Self::new() } }
