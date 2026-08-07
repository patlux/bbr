//! Response-legality and lifecycle contract tests: the exhaustive kind ×
//! response matrix, the exhaustive state × event matrix, exact payload and
//! rejection assertions, and the security invariants (cancellation/
//! supersession never become credential use, server trust is reject-only, no
//! secret-shaped material in Debug output).

mod debug;
mod lifecycle;
mod response_matrix;
mod support;
mod transition_matrix;
