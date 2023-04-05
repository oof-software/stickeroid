use std::fmt::Display;

use log::{error, info};

pub trait ExpectLog {
    type Output;
    fn expect_log(self, msg: &str) -> Self::Output;
    fn expect_log_fmt(self, f: impl FnOnce() -> String) -> Self::Output;
    fn expect_log_v(self, err_msg: &str, ok_msg: &str) -> Self::Output;
    fn expect_log_v_fmt(
        self,
        err_f: impl FnOnce() -> String,
        ok_f: impl FnOnce(&Self::Output) -> String,
    ) -> Self::Output;
}

impl<T> ExpectLog for Option<T> {
    type Output = T;
    fn expect_log(self, msg: &str) -> Self::Output {
        match self {
            None => {
                error!("{msg}: unwrapped empty optional");
                panic!("{msg}: unwrapped empty optional");
            }
            Some(v) => v,
        }
    }
    fn expect_log_fmt(self, f: impl FnOnce() -> String) -> Self::Output {
        match self {
            None => {
                let msg = f();
                error!("{msg}: unwrapped empty optional");
                panic!("{msg}: unwrapped empty optional");
            }
            Some(v) => v,
        }
    }
    fn expect_log_v(self, err_msg: &str, ok_msg: &str) -> Self::Output {
        match self {
            None => {
                error!("{err_msg}: unwrapped empty optional");
                panic!("{err_msg}: unwrapped empty optional");
            }
            Some(v) => {
                info!("{ok_msg}");
                v
            }
        }
    }
    fn expect_log_v_fmt(
        self,
        err_f: impl FnOnce() -> String,
        ok_f: impl FnOnce(&Self::Output) -> String,
    ) -> Self::Output {
        match self {
            None => {
                let err_msg = err_f();
                error!("{err_msg}: unwrapped empty optional");
                panic!("{err_msg}: unwrapped empty optional");
            }
            Some(v) => {
                let ok_msg = ok_f(&v);
                info!("{ok_msg}");
                v
            }
        }
    }
}

impl<T, E> ExpectLog for Result<T, E>
where
    E: Display,
{
    type Output = T;
    fn expect_log(self, msg: &str) -> Self::Output {
        match self {
            Err(e) => {
                error!("{msg}: {e}");
                panic!("{msg}: {e}");
            }
            Ok(v) => v,
        }
    }
    fn expect_log_fmt(self, f: impl FnOnce() -> String) -> Self::Output {
        match self {
            Err(e) => {
                let msg = f();
                error!("{msg}: {e}");
                panic!("{msg}: {e}");
            }
            Ok(v) => v,
        }
    }
    fn expect_log_v(self, err_msg: &str, ok_msg: &str) -> Self::Output {
        match self {
            Err(e) => {
                error!("{err_msg}: {e}");
                panic!("{err_msg}: {e}");
            }
            Ok(v) => {
                info!("{ok_msg}");
                v
            }
        }
    }
    fn expect_log_v_fmt(
        self,
        err_f: impl FnOnce() -> String,
        ok_f: impl FnOnce(&Self::Output) -> String,
    ) -> Self::Output {
        match self {
            Err(e) => {
                let err_msg = err_f();
                error!("{err_msg}: {e}");
                panic!("{err_msg}: {e}");
            }
            Ok(v) => {
                let ok_msg = ok_f(&v);
                info!("{ok_msg}");
                v
            }
        }
    }
}
