#![allow(non_snake_case)]
#![feature(io_error_more)]

use std::ops::{Deref, DerefMut};
use std::sync::Mutex;

use jni::objects::{JObject, JValue};
use jni::sys::jint;
use jni::JNIEnv;
use mumble_link::{ErrorCode, MumbleLink, Position};
use mut_static::MutStatic;

type JniResult<T = ()> = std::result::Result<T, jni::errors::Error>;

const MUMBLE_VEC: &str = "Lcom/moonsworth/client/mumble/MumbleVec;";
const NAME: &str = "Minecraft";
const DESC: &str = "Minecraft (1.8.9)";

type Link = Mutex<Result<MumbleLink, ErrorCode>>;

lazy_static::lazy_static! {
    static ref INSTANCE: MutStatic<Link> = MutStatic::from(Mutex::new(MumbleLink::new(NAME, DESC)));
}

fn reset_link(name: &str, desc: &str) -> Link {
    let link = Mutex::new(MumbleLink::new(name, desc));
    std::mem::replace(INSTANCE.write().unwrap().deref_mut(), link)
}

#[no_mangle]
pub extern "system" fn Java_com_moonsworth_client_mumble_MumbleLink_init(
    env: JNIEnv,
    _input: JObject,
) -> jint {
    eprintln!("CALLED Java_com_moonsworth_client_mumble_MumbleLink_init");
    // TODO: Take name from user
    match reset_link(NAME, DESC).lock().unwrap().deref() {
        Ok(_) => 0,
        Err(e) => {
            let code = (*e) as i32;
            eprintln!("MUMBLE ERROR: {}", e);

            let _ = popup(
                env,
                "Mumble Link",
                "Mumble link failed to connect. Is Mumble open?",
            );

            -code
        }
    }
}

pub fn popup(env: JNIEnv, name: &str, desc: &str) -> Result<(), jni::errors::Error> {
    let name = env.new_string(name)?.into();
    let desc = env.new_string(desc)?.into();

    let accessor = env
        .get_static_field(
            "com/grappenmaker/solarpatcher/util/generation/Accessors$Utility",
            "INSTANCE",
            "Lcom/grappenmaker/solarpatcher/util/generation/Accessors$Utility;",
        )?
        .l()?;

    env.call_method(
        accessor,
        "displayPopup",
        "(Ljava/lang/String;Ljava/lang/String;)V",
        &[JValue::Object(name), JValue::Object(desc)],
    )?;

    Ok(())
}

#[no_mangle]
pub extern "system" fn Java_com_moonsworth_client_mumble_MumbleLink_update(
    env: JNIEnv,
    _this: JObject,
    input: JObject,
) {
    eprintln!("CALLED Java_com_moonsworth_client_mumble_MumbleLink_update");
    // This is error handling hell.
    let link = INSTANCE.read();
    if link.is_err() {
        eprintln!("Mumble Error: Link wasn't initialized (code -1)");
        return;
    }
    let link = link.unwrap();
    let link = link.lock();
    if link.is_err() {
        eprintln!("Mumble Error: Unable to lock link (code -1)");
        return;
    }
    let mut link = link.unwrap();
    if let Err(i) = link.deref() {
        let code = (*i) as i32;
        eprintln!("Mumble Error: {} (code {})", i, code);
        return;
    }

    let link = link.as_mut().unwrap();

    let avatar_front = env.get_field(input, "avatarFront", MUMBLE_VEC).unwrap();
    let avatar_position = env.get_field(input, "avatarPosition", MUMBLE_VEC).unwrap();
    let avatar_top = env.get_field(input, "avatarTop", MUMBLE_VEC).unwrap();

    let camera_front = env.get_field(input, "cameraFront", MUMBLE_VEC).unwrap();
    let camera_position = env.get_field(input, "cameraPosition", MUMBLE_VEC).unwrap();
    let camera_top = env.get_field(input, "cameraTop", MUMBLE_VEC).unwrap();

    let avatar = into_pos(&env, avatar_front, avatar_top, avatar_position).expect("INVALIDE");
    let camera = into_pos(&env, camera_front, camera_top, camera_position).expect("INVALIDE");

    link.update(avatar, camera);
}

fn mumble_vec_to_array(env: &JNIEnv, input: JValue) -> JniResult<[f32; 3]> {
    let obj = input.l()?;

    let x = env.get_field(obj, "xCoord", "D")?.d()? as f32;
    let y = env.get_field(obj, "yCoord", "D")?.d()? as f32;
    let z = env.get_field(obj, "zCoord", "D")?.d()? as f32;

    Ok([x, y, z])
}

fn into_pos(env: &JNIEnv, front: JValue, top: JValue, position: JValue) -> JniResult<Position> {
    let front = mumble_vec_to_array(env, front)?;
    let top = mumble_vec_to_array(env, top)?;
    let position = mumble_vec_to_array(env, position)?;

    Ok(Position {
        front,
        top,
        position,
    })
}
