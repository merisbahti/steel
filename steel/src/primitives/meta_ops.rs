use crate::env::VOID;
use crate::gc::{get_object_count, Gc};
use crate::rerrs::SteelErr;
use crate::rvals::{Result, SteelVal};
use crate::stop;

use futures::{executor::LocalPool, future::join_all};

use async_compat::Compat;

use std::cell::RefCell;

pub struct MetaOperations {}
impl MetaOperations {
    pub fn inspect_bytecode() -> SteelVal {
        SteelVal::FuncV(|args: &[Gc<SteelVal>]| -> Result<Gc<SteelVal>> {
            // let mut error_message = String::new();

            if args.len() == 1 {
                if let SteelVal::Closure(bytecode_lambda) = args[0].as_ref() {
                    crate::core::instructions::pretty_print_dense_instructions(
                        &bytecode_lambda.body_exp(),
                    );
                    Ok(VOID.with(|f| Gc::clone(f)))
                } else {
                    stop!(TypeMismatch => "inspect-bytecode expects a closure object");
                }
            } else {
                stop!(ArityMismatch => "inspect-bytecode takes only one argument");
            }
        })
    }

    pub fn active_objects() -> SteelVal {
        SteelVal::FuncV(|args: &[Gc<SteelVal>]| -> Result<Gc<SteelVal>> {
            if args.len() != 0 {
                stop!(ArityMismatch => "active-object-count expects only one argument");
            }
            Ok(Gc::new(SteelVal::IntV(get_object_count() as isize)))
        })
    }

    pub fn memory_address() -> SteelVal {
        SteelVal::FuncV(|args: &[Gc<SteelVal>]| -> Result<Gc<SteelVal>> {
            if args.len() != 1 {
                stop!(ArityMismatch => "memory address takes one address")
            }

            let memory_address = format!("{:p}", &args[0].as_ptr());

            Ok(Gc::new(SteelVal::StringV(memory_address.into())))
        })
    }

    pub fn assert_truthy() -> SteelVal {
        SteelVal::FuncV(|args: &[Gc<SteelVal>]| -> Result<Gc<SteelVal>> {
            if args.len() != 1 {
                stop!(ArityMismatch => "assert takes one argument")
            }
            if let SteelVal::BoolV(true) = &args[0].as_ref() {
                Ok(Gc::new(SteelVal::Void))
            } else {
                panic!("Value given not true!")
            }
        })
    }

    // TODO
    pub fn new_box() -> SteelVal {
        SteelVal::FuncV(|args: &[Gc<SteelVal>]| -> Result<Gc<SteelVal>> {
            if args.len() != 1 {
                stop!(ArityMismatch => "box takes one argument")
            }

            Ok(Gc::new(SteelVal::BoxV(Gc::new(RefCell::new(Gc::clone(
                &args[0],
            ))))))
        })
    }

    // TODO
    pub fn unbox() -> SteelVal {
        SteelVal::FuncV(|args: &[Gc<SteelVal>]| -> Result<Gc<SteelVal>> {
            if args.len() != 1 {
                stop!(ArityMismatch => "unbox takes one argument")
            }
            if let SteelVal::BoxV(inner) = &args[0].as_ref() {
                Ok(inner.unwrap().into_inner())
            } else {
                stop!(TypeMismatch => "unbox takes a box")
            }
        })
    }

    // TODO
    pub fn set_box() -> SteelVal {
        SteelVal::FuncV(|args: &[Gc<SteelVal>]| -> Result<Gc<SteelVal>> {
            if args.len() != 2 {
                stop!(ArityMismatch => "setbox! takes two arguments")
            }
            if let SteelVal::BoxV(inner) = &args[0].as_ref() {
                Ok(inner.replace(Gc::clone(&args[1])))
            } else {
                stop!(TypeMismatch => "setbox! takes a box")
            }
        })
    }

    // Uses a generic executor w/ the compat struct in order to allow tokio ecosystem functions inside
    // the interpreter
    pub fn exec_async() -> SteelVal {
        SteelVal::FuncV(|args: &[Gc<SteelVal>]| -> Result<Gc<SteelVal>> {
            let mut executor = LocalPool::new();

            let joined_futures: Vec<_> = args
                .into_iter()
                .map(|x| {
                    if let SteelVal::FutureV(f) = x.as_ref() {
                        Ok(f.unwrap().into_shared())
                    } else {
                        stop!(TypeMismatch => "exec-async given non future")
                    }
                })
                .collect::<Result<Vec<_>>>()?;

            let futures = join_all(joined_futures);

            // spawner.spawn_local_obj(joined_futures);

            // let future = LocalFutureObj::new(Box::pin(async {}));
            // spawner.spawn_local_obj(future);
            // executor.run_until(future);
            Ok(Gc::new(SteelVal::VectorV(Gc::new(
                executor
                    .run_until(Compat::new(futures))
                    .into_iter()
                    .collect::<Result<_>>()?,
            ))))

            // unimplemented!()
        })
    }

    // pub fn tokio_exec() -> SteelVal {
    //     SteelVal::FuncV(|args: &[Gc<SteelVal>]| -> Result<Gc<SteelVal>> {
    //         // let mut executor = LocalPool::new();

    //         let mut basic_rt = runtime::Builder::new()
    //             .basic_scheduler()
    //             .enable_all()
    //             .build()
    //             .unwrap();

    //         // let ret = Builder::new().threaded_scheduler().enable_all().build();

    //         let joined_futures: Vec<_> = args
    //             .into_iter()
    //             .map(|x| {
    //                 if let SteelVal::FutureV(f) = x.as_ref() {
    //                     Ok(f.clone().into_shared())
    //                 } else {
    //                     stop!(TypeMismatch => "exec-async given non future")
    //                 }
    //             })
    //             .collect::<Result<Vec<_>>>()?;

    //         let futures = join_all(joined_futures);

    //         // spawner.spawn_local_obj(joined_futures);

    //         // let future = LocalFutureObj::new(Box::pin(async {}));
    //         // spawner.spawn_local_obj(future);
    //         // executor.run_until(future);
    //         Ok(Gc::new(SteelVal::VectorV(
    //             basic_rt
    //                 .block_on(futures)
    //                 .into_iter()
    //                 .collect::<Result<_>>()?,
    //         )))

    //         // unimplemented!()
    //     })
    // }
}
