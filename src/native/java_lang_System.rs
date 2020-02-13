#![allow(non_snake_case)]

use crate::classfile::types::BytesRef;
use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc, OopRef};
use crate::runtime::{self, JavaThread};
use crate::runtime::{require_class3, JavaCall, Stack};
use crate::util;
use std::sync::{Arc, Mutex};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn(
            "arraycopy",
            "(Ljava/lang/Object;ILjava/lang/Object;II)V",
            Box::new(jvm_arraycopy),
        ),
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn(
            "initProperties",
            "(Ljava/util/Properties;)Ljava/util/Properties;",
            Box::new(jvm_initProperties),
        ),
        new_fn("setIn0", "(Ljava/io/InputStream;)V", Box::new(jvm_setIn0)),
        new_fn("setOut0", "(Ljava/io/PrintStream;)V", Box::new(jvm_setOut0)),
        new_fn("setErr0", "(Ljava/io/PrintStream;)V", Box::new(jvm_setErr0)),
        //Note: just for debug
        //        new_fn("getProperty", "(Ljava/lang/String;)Ljava/lang/String;", Box::new(jvm_getProperty)),
    ]
}

fn jvm_registerNatives(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_arraycopy(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let src = args.get(0).unwrap();
    let src_pos = {
        let arg1 = args.get(1).unwrap();
        let arg1 = arg1.lock().unwrap();
        match arg1.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };
    let dest = args.get(2).unwrap();
    let dest_pos = {
        let arg3 = args.get(3).unwrap();
        let arg3 = arg3.lock().unwrap();
        match arg3.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };
    let length = {
        let arg4 = args.get(4).unwrap();
        let arg4 = arg4.lock().unwrap();
        match arg4.v {
            Oop::Int(v) => v,
            _ => unreachable!(),
        }
    };

    //todo: do check & throw exception

    if length == 0 {
        return Ok(None);
    }

    let is_str = util::oop::is_str(src.clone());
    if is_str {
        let src: Vec<OopRef> = {
            let src = src.lock().unwrap();
            match &src.v {
                Oop::Str(s) => {
                    //just construct the needed region
                    s[src_pos as usize..(src_pos + length - 1) as usize]
                        .iter()
                        .map(|v| OopDesc::new_int(*v as i32))
                        .collect()
                }
                _ => unreachable!(),
            }
        };

        let mut dest = dest.lock().unwrap();
        match &mut dest.v {
            Oop::Array(dest) => {
                dest.elements[dest_pos as usize..(dest_pos + length - 1) as usize]
                    .clone_from_slice(&src[..]);
            }
            _ => unreachable!(),
        }
    } else {
        //这里为了避免clone一个大数组，做了优化
        //src 和 dest 有可能是同一个对象，所以，两个不能同时上锁
        if Arc::ptr_eq(&src, &dest) {
            let src = {
                let ary = src.lock().unwrap();
                match &ary.v {
                    Oop::Array(ary) => {
                        let mut new_ary = Vec::new();
                        new_ary.clone_from_slice(
                            &ary.elements[src_pos as usize..(src_pos + length - 1) as usize],
                        );
                        new_ary
                    }
                    _ => unreachable!(),
                }
            };

            let mut dest = dest.lock().unwrap();
            match &mut dest.v {
                Oop::Array(ary) => {
                    ary.elements[dest_pos as usize..(dest_pos + length - 1) as usize]
                        .clone_from_slice(&src[..]);
                }
                _ => unreachable!(),
            }
        } else {
            let src = src.lock().unwrap();
            match &src.v {
                Oop::Array(src) => {
                    let mut dest = dest.lock().unwrap();
                    match &mut dest.v {
                        Oop::Array(dest) => {
                            dest.elements[dest_pos as usize..(dest_pos + length - 1) as usize]
                                .clone_from_slice(
                                    &src.elements
                                        [src_pos as usize..(src_pos + length - 1) as usize],
                                );
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(None)
}

fn jvm_initProperties(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    //fixme:
    let props = vec![
        ("java.specification.version", "1.8"),
        ("java.specification.name", "Java Platform API Specification"),
        ("java.specification.vendor", "Chuan"),
        ("java.version", "1.8"),
        ("java.vendor", "Chuan"),
        ("java.vendor.url", "https://github.com/douchuan/jvm"),
        ("java.vendor.url.bug", "https://github.com/douchuan/jvm"),
        ("java.class.version", "52.0"),
        ("os.name", "Mac OS X"),
        ("os.version", "18.7.0"),
        ("os.arch", "x86"),
        ("file.separator", util::PATH_DELIMITER_STR),
        ("path.separator", util::PATH_SEP_STR),
        ("line.separator", "\n"),
        ("user.language", "en"),
        ("file.encoding", "UTF-8"),
        ("sun.jnu.encoding", "UTF-8"),
        ("file.encoding.pkg", "sun.io"),
        ("sun.io.unicode.encoding", "UnicodeBig"),
        ("sun.cpu.isalist", ""),
        ("sun.cpu.endian", "little"),
        ("sun.arch.data.model", "32"),
        ("user.name", "chuan"),
        ("user.home", "/Users/douchuan/"),
        ("user.dir", "/Users/douchuan/"),
        ("java.home", "/Users/douchuan/work/prj_rust/jvm/test/"),
        ("sun.stdout.encoding", "UTF-8"),
        ("sun.stderr.encoding", "UTF-8"),
    ];

    let props: Vec<(OopRef, OopRef)> = props
        .iter()
        .map(|(k, v)| {
            let k = Vec::from(k.as_bytes());
            let k = new_ref!(k);
            let k = OopDesc::new_str(k);

            let v = Vec::from(v.as_bytes());
            let v = new_ref!(v);
            let v = OopDesc::new_str(v);
            (k, v)
        })
        .collect();

    let props_oop = args.get(0).unwrap();
    let cls = {
        let v = props_oop.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };

    let mir = {
        let cls = cls.lock().unwrap();
        let id = util::new_method_id(
            b"put",
            b"(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
        );
        cls.get_virtual_method(id).unwrap()
    };

    for it in props.iter() {
        let args = vec![props_oop.clone(), it.0.clone(), it.1.clone()];

        let mut jc = JavaCall::new_with_args(jt, mir.clone(), args);
        let mut stack = runtime::Stack::new(1);
        jc.invoke(jt, &mut stack, false);

        //fixme: should be removed
        if jt.is_meet_ex() {
            unreachable!("jvm_initProperties meet ex");
        }
    }

    Ok(Some(props_oop.clone()))
}

fn jvm_setIn0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"in", b"Ljava/io/InputStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_setOut0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"out", b"Ljava/io/PrintStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_setErr0(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let v = args.get(0).unwrap();
    let cls = { env.lock().unwrap().class.clone() };
    let mut cls = cls.lock().unwrap();
    let id = cls.get_field_id(b"err", b"Ljava/io/PrintStream;", true);
    cls.put_static_field_value(id, v.clone());
    Ok(None)
}

fn jvm_getProperty(jt: &mut JavaThread, env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let key = args.get(0).unwrap();

    let str_key = util::oop::extract_str(key.clone());
    warn!(
        "xxxx jvm_getProperty key = {}",
        String::from_utf8_lossy(str_key.as_slice())
    );

    let cls = require_class3(None, b"java/lang/System").unwrap();
    let props = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"props", b"Ljava/util/Properties;", true);
        cls.get_static_field_value(id)
    };

    let prop_cls = require_class3(None, b"java/util/Properties").unwrap();
    let mir = {
        let cls = prop_cls.lock().unwrap();
        let id = util::new_method_id(b"getProperty", b"(Ljava/lang/String;)Ljava/lang/String;");
        cls.get_class_method(id).unwrap()
    };

    let args = vec![props, key.clone()];
    let mut stack = Stack::new(1);
    let mut jc = runtime::java_call::JavaCall::new_with_args(jt, mir, args);
    jc.invoke(jt, &mut stack, false);

    let v = stack.pop_ref();

    //    trace!("xxxxx 1, str_key = {}", String::from_utf8_lossy(str_key.as_slice()));
    //    let str_v = util::oop::extract_str(v.clone());
    //    warn!("xxxx jvm_getProperty v = {}", String::from_utf8_lossy(str_v.as_slice()));
    //    trace!("xxxxx 2");

    Ok(Some(v))
}
