#![allow(non_snake_case)]
#![feature(io_error_more)]

use std::sync::Mutex;

use jni::objects::{JObject, JValue};
use jni::sys::jint;
use jni::JNIEnv;
use mumble_link::{ErrorCode, MumbleLink, Position};
use once_cell::sync::OnceCell;

type JniResult<T = ()> = std::result::Result<T, jni::errors::Error>;

const MUMBLE_VEC: &str = "Lcom/moonsworth/client/mumble/MumbleVec;";
const NAME: &str = "Solar Tweaks";
const DESC: &str = "Tweaked Minecraft";

static INSTANCE: OnceCell<Mutex<MumbleLink>> = OnceCell::new();

fn get_link() -> Result<&'static Mutex<MumbleLink>, ErrorCode> {
    _create_link(NAME, DESC)
}

fn create_link(name: Option<&str>, desc: Option<&str>) -> Result<(), ErrorCode> {
    _create_link(name.unwrap_or(NAME), desc.unwrap_or(DESC)).map(|_| ())
}

fn _create_link(name: &str, desc: &str) -> Result<&'static Mutex<MumbleLink>, ErrorCode> {
    // If there is already a link, don't create one or throw an error
    // Only error when the link failed to initialize.
    INSTANCE.get_or_try_init(|| Ok(Mutex::new(MumbleLink::new(name, desc)?)))
}

#[no_mangle]
pub extern "system" fn Java_com_moonsworth_client_mumble_MumbleLink_init(
    env: JNIEnv,
    _input: JObject,
) -> jint {
    // TODO: Take name from user
    match create_link(None, None) {
        Ok(_) => 0,
        Err(e) => {
            let code = e as i32;
            eprintln!("MUMBLE ERROR: {}", e);

            let _ = popup(
                env,
                "Mumble Link",
                "Mumble link failed to connect. Is Mumble open?",
            );

            return code;
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
    let link = get_link();

    if let Err(i) = link {
        let code = i as i32;
        eprintln!("Mumble Error: {} (code {})", i, code);
        return;
    }

    let mut link = link.unwrap().lock().expect("Could not lock link");

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
    let front = mumble_vec_to_array(&env, front)?;
    let top = mumble_vec_to_array(&env, top)?;
    let position = mumble_vec_to_array(&env, position)?;

    Ok(Position {
        front,
        top,
        position,
    })
}
