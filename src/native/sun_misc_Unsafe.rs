#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{Oop, OopDesc};
use crate::runtime::{require_class3, JavaThread};
use crate::types::OopRef;
use crate::util;
use std::os::raw::c_void;

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn(
            "arrayBaseOffset",
            "(Ljava/lang/Class;)I",
            Box::new(jvm_arrayBaseOffset),
        ),
        new_fn(
            "arrayIndexScale",
            "(Ljava/lang/Class;)I",
            Box::new(jvm_arrayIndexScale),
        ),
        new_fn("addressSize", "()I", Box::new(jvm_addressSize)),
        new_fn(
            "objectFieldOffset",
            "(Ljava/lang/reflect/Field;)J",
            Box::new(jvm_objectFieldOffset),
        ),
        new_fn(
            "compareAndSwapObject",
            "(Ljava/lang/Object;JLjava/lang/Object;Ljava/lang/Object;)Z",
            Box::new(jvm_compareAndSwapObject),
        ),
        new_fn(
            "getIntVolatile",
            "(Ljava/lang/Object;J)I",
            Box::new(jvm_getIntVolatile),
        ),
        new_fn(
            "compareAndSwapInt",
            "(Ljava/lang/Object;JII)Z",
            Box::new(jvm_compareAndSwapInt),
        ),
        new_fn("allocateMemory", "(J)J", Box::new(jvm_allocateMemory)),
        new_fn("freeMemory", "(J)V", Box::new(jvm_freeMemory)),
        new_fn("putLong", "(JJ)V", Box::new(jvm_putLong)),
        new_fn("getByte", "(J)B", Box::new(jvm_getByte)),
        new_fn(
            "compareAndSwapLong",
            "(Ljava/lang/Object;JJJ)Z",
            Box::new(jvm_compareAndSwapLong),
        ),
        new_fn(
            "getObjectVolatile",
            "(Ljava/lang/Object;J)Ljava/lang/Object;",
            Box::new(jvm_getObjectVolatile),
        ),
        new_fn("pageSize", "()I", Box::new(jvm_pageSize)),
        new_fn(
            "getLongVolatile",
            "(Ljava/lang/Object;J)J",
            Box::new(jvm_getLongVolatile),
        ),
        new_fn(
            "setMemory",
            "(Ljava/lang/Object;JJB)V",
            Box::new(jvm_setMemory),
        ),
        new_fn("putChar", "(JC)V", Box::new(jvm_putChar)),
    ]
}

fn jvm_registerNatives(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(None)
}

fn jvm_arrayBaseOffset(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_int(0)))
}

fn jvm_arrayIndexScale(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    //    let v = std::mem::size_of::<*mut u8>();
    //    Ok(Some(OopDesc::new_int(v as i32)))
    Ok(Some(OopDesc::new_int(1)))
}

fn jvm_addressSize(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    let v = std::mem::size_of::<*mut u8>();
    Ok(Some(OopDesc::new_int(v as i32)))
}

fn jvm_objectFieldOffset(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let field = args[1].clone();

    {
        let v = field.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => {
                let cls = inst.class.clone();
                let cls = cls.lock().unwrap();
                assert_eq!(cls.name.as_slice(), b"java/lang/reflect/Field");
            }
            _ => unreachable!(),
        }
    }

    let cls = require_class3(None, b"java/lang/reflect/Field").unwrap();
    let v = {
        let cls = cls.lock().unwrap();
        let id = cls.get_field_id(b"slot", b"I", false);
        cls.get_field_value(field, id)
    };

    let v = util::oop::extract_int(v);

    Ok(Some(OopDesc::new_long(v as i64)))
}

//fixme: 此处语义上要求是原子操作，这里需要重新实现
fn jvm_compareAndSwapObject(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap().clone());
    let old_data = args.get(3).unwrap();
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let v = owner.lock().unwrap();
        match &v.v {
            Oop::Mirror(mirror) => mirror.field_values[offset as usize].clone(),
            Oop::Array(ary) => ary.elements[offset as usize].clone(),
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            t => unreachable!("{:?}", t),
        }
    };

    if util::oop::if_acmpeq(v_at_offset, old_data.clone()) {
        let mut v = owner.lock().unwrap();
        match &mut v.v {
            Oop::Mirror(mirror) => {
                mirror.field_values[offset as usize] = new_data.clone();
            }
            Oop::Array(ary) => {
                ary.elements[offset as usize] = new_data.clone();
            }
            Oop::Inst(inst) => inst.field_values[offset as usize] = new_data.clone(),
            _ => unreachable!(),
        }

        Ok(Some(OopDesc::new_int(1)))
    } else {
        Ok(Some(OopDesc::new_int(0)))
    }
}

fn jvm_getIntVolatile(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap().clone());
    let v_at_offset = {
        let v = owner.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        }
    };
    Ok(Some(v_at_offset))
}

fn jvm_compareAndSwapInt(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap().clone());
    let old_data = util::oop::extract_int(args.get(3).unwrap().clone());
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let v = owner.lock().unwrap();
        let v = match &v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        };

        util::oop::extract_int(v)
    };

    if v_at_offset == old_data {
        let mut v = owner.lock().unwrap();
        match &mut v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize] = new_data.clone(),
            _ => unreachable!(),
        }

        Ok(Some(OopDesc::new_int(1)))
    } else {
        Ok(Some(OopDesc::new_int(0)))
    }
}

fn jvm_allocateMemory(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let size = util::oop::extract_long(args.get(1).unwrap().clone()) as usize;
    let arr = unsafe { libc::malloc(std::mem::size_of::<u8>() * size) };
    let v = arr as i64;

    Ok(Some(OopDesc::new_long(v)))
}

fn jvm_freeMemory(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let ptr = util::oop::extract_long(args.get(1).unwrap().clone()) as *mut libc::c_void;

    unsafe {
        libc::free(ptr);
    }

    Ok(None)
}

fn jvm_putLong(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let ptr = util::oop::extract_long(args.get(1).unwrap().clone()) as *mut libc::c_void;
    let l = util::oop::extract_long(args.get(2).unwrap().clone());
    let v = l.to_be_bytes();
    let v = vec![v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]];
    unsafe {
        libc::memcpy(ptr, v.as_ptr() as *const c_void, 8);
    }

    Ok(None)
}

fn jvm_getByte(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let ptr = util::oop::extract_long(args.get(1).unwrap().clone()) as *const u8;
    let v = unsafe { *ptr };
    Ok(Some(OopDesc::new_int(v as i32)))
}

fn jvm_compareAndSwapLong(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap().clone());
    let old_data = util::oop::extract_long(args.get(3).unwrap().clone());
    let new_data = args.get(4).unwrap();

    let v_at_offset = {
        let v = owner.lock().unwrap();
        let v = match &v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        };

        util::oop::extract_long(v)
    };

    if v_at_offset == old_data {
        let mut v = owner.lock().unwrap();
        match &mut v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize] = new_data.clone(),
            _ => unreachable!(),
        }

        Ok(Some(OopDesc::new_int(1)))
    } else {
        Ok(Some(OopDesc::new_int(0)))
    }
}

fn jvm_getObjectVolatile(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap().clone());
    let v_at_offset = {
        let v = owner.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            Oop::Array(ary) => ary.elements[offset as usize].clone(),
            t => unreachable!("t = {:?}", t),
        }
    };
    Ok(Some(v_at_offset))
}

fn jvm_pageSize(_jt: &mut JavaThread, _env: JNIEnv, _args: Vec<OopRef>) -> JNIResult {
    Ok(Some(OopDesc::new_int(4 * 1024)))
}

fn jvm_getLongVolatile(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let owner = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap().clone());
    let v_at_offset = {
        let v = owner.lock().unwrap();
        match &v.v {
            Oop::Inst(inst) => inst.field_values[offset as usize].clone(),
            _ => unreachable!(),
        }
    };
    Ok(Some(v_at_offset))
}

fn jvm_setMemory(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let _this = args.get(0).unwrap();
    let obj = args.get(1).unwrap();
    let offset = util::oop::extract_long(args.get(2).unwrap().clone());
    let size = util::oop::extract_long(args.get(3).unwrap().clone());
    let value = util::oop::extract_int(args.get(4).unwrap().clone());

    let dest = {
        let v = obj.lock().unwrap();
        match &v.v {
            Oop::Null => offset as *mut libc::c_void,
            Oop::Inst(_inst) => unimplemented!("inst"),
            _ => unimplemented!(),
        }
    };

    unsafe {
        libc::memset(dest, value, size as usize);
    }

    Ok(None)
}

fn jvm_putChar(_jt: &mut JavaThread, _env: JNIEnv, args: Vec<OopRef>) -> JNIResult {
    let dest = util::oop::extract_long(args.get(1).unwrap().clone()) as *mut libc::c_void;
    let value = util::oop::extract_int(args.get(2).unwrap().clone());

    unsafe {
        libc::memset(dest, value, 1);
    }

    Ok(None)
}
