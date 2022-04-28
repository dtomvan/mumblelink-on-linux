//! **mumble-link** provides an API for using the [Mumble Link][link] plugin
//! for position-aware VoIP communications.
//!
//! [link]: https://wiki.mumble.info/wiki/Link
//!
//! Connect to Mumble link with `MumbleLink::new()`, set the context or player
//! identity as needed, and call `update()` every frame with the position data.

extern crate libc;
extern crate winapi;

use libc::{c_float, wchar_t};
use std::{fmt::Display, io, mem, ptr};

#[cfg_attr(not(windows), allow(unused_macros))]
macro_rules! wide {
    ($($ch:ident)*) => {
        [$(stringify!($ch).as_bytes()[0] as ::libc::wchar_t,)* 0]
    }
}

/// The error to send to mumble.
/// See: https://www.mumble.info/documentation/developer/positional-audio/link-plugin/
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ErrorCode {
    Success = 0,
    OpenFileMappingW = 1,
    MapViewOfFile = 2,
    ShmOpen = 3,
    MMap = 4,
    NoMem = 5,
    Unknown = 6,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if let Self::Success = self {
            "no error"
        } else if let Self::NoMem = self {
            "shared memory was not initialized"
        } else if let Self::Unknown = self {
            "unknown Error"
        } else if cfg!(windows) {
            match self {
                ErrorCode::OpenFileMappingW => "OpenFileMappingW failed to return a handle",
                ErrorCode::MapViewOfFile => "MapViewOfFile failed to return a structure",
                _ => unreachable!(),
            }
        } else if cfg!(unix) {
            match self {
                ErrorCode::ShmOpen => "shm_open returned a negative integer",
                ErrorCode::MMap => "mmap failed to return a structure",
                _ => unreachable!(),
            }
        } else {
            unreachable!()
        })
    }
}

#[cfg_attr(windows, path = "windows.rs")]
#[cfg_attr(not(windows), path = "unix.rs")]
mod imp;

/// A position in three-dimensional space.
///
/// The vectors are in a left-handed coordinate system: X positive towards
/// "right", Y positive towards "up", and Z positive towards "front". One unit
/// is treated as one meter by the sound engine.
///
/// `front` and `top` should be unit vectors and perpendicular to each other.
#[derive(Copy, Clone, Debug)]
pub struct Position {
    /// The character's position in space.
    pub position: [f32; 3],
    /// A unit vector pointing out of the character's eyes.
    pub front: [f32; 3],
    /// A unit vector pointing out of the top of the character's head.
    pub top: [f32; 3],
}

// `f32` is used above for tidyness; assert that it matches c_float.
const _ASSERT_CFLOAT_IS_F32: c_float = 0f32;

impl Default for Position {
    fn default() -> Self {
        Position {
            position: [0., 0., 0.],
            front: [0., 0., 1.],
            top: [0., 1., 0.],
        }
    }
}

#[derive(Copy, Debug)]
struct LinkedMem {
    #[cfg(windows)]
    ui_version: winapi::UINT32,
    #[cfg(windows)]
    ui_tick: winapi::DWORD,
    #[cfg(not(windows))]
    ui_version: u32,
    #[cfg(not(windows))]
    ui_tick: u32,
    avatar: Position,
    name: [wchar_t; 256],
    camera: Position,
    identity: [wchar_t; 256],
    #[cfg(windows)]
    context_len: winapi::UINT32,
    #[cfg(not(windows))]
    context_len: u32,
    context: [u8; 256],
    description: [wchar_t; 2048],
}

impl Clone for LinkedMem {
    fn clone(&self) -> Self {
        *self
    }
}

impl LinkedMem {
    fn new(name: &str, description: &str) -> LinkedMem {
        let mut result = LinkedMem {
            ui_version: 2,
            ui_tick: 0,
            avatar: Position::default(),
            name: [0; 256],
            camera: Position::default(),
            identity: [0; 256],
            context_len: 0,
            context: [0; 256],
            description: [0; 2048],
        };
        imp::copy(&mut result.name, name);
        imp::copy(&mut result.description, description);
        result
    }

    fn set_context(&mut self, context: &[u8]) {
        let len = std::cmp::min(context.len(), self.context.len());
        self.context[..len].copy_from_slice(&context[..len]);
        self.context_len = len as u32;
    }

    #[inline]
    fn set_identity(&mut self, identity: &str) {
        imp::copy(&mut self.identity, identity);
    }

    fn update(&mut self, avatar: Position, camera: Position) {
        self.ui_tick = self.ui_tick.wrapping_add(1);
        self.avatar = avatar;
        self.camera = camera;
    }
}

macro_rules! docs {
    ($(#[$attr:meta])* pub fn set_context(&mut $s:ident, $c:ident: &[u8]) $b:block) => {
        /// Update the context string, used to determine which users on a Mumble
        /// server should hear each other positionally.
        ///
        /// If context between two Mumble users does not match, the positional audio
        /// data is stripped server-side and voice will be received as
        /// non-positional. Accordingly, the context should only match for players
        /// on the same game, server, and map, depending on the game itself. When
        /// in doubt, err on the side of including less; this allows for more
        /// flexibility in the future.
        ///
        /// The context should be changed infrequently, at most a few times per
        /// second.
        ///
        /// The context has a maximum length of 256 bytes.
        $(#[$attr])*
        pub fn set_context(&mut $s, $c: &[u8]) $b
    };
    ($(#[$attr:meta])* pub fn set_identity(&mut $s:ident, $i:ident: &str) $b:block) => {
        /// Update the identity, uniquely identifying the player in the given
        /// context. This is usually the in-game name or ID.
        ///
        /// The identity may also contain any additional information about the
        /// player which might be useful for the Mumble server, for example to move
        /// teammates to the same channel or give squad leaders additional powers.
        /// It is recommended that a parseable format like JSON or CSV is used for
        /// this.
        ///
        /// The identity should be changed infrequently, at most a few times per
        /// second.
        ///
        /// The identity has a maximum length of 255 UTF-16 code units.
        $(#[$attr])*
        pub fn set_identity(&mut $s, $i: &str) $b
    };
    ($(#[$attr:meta])* pub fn update(&mut $s:ident, $a:ident: Position, $c:ident: Position) $b:block) => {
        /// Update the link with the latest position information. Should be called
        /// once per frame.
        ///
        /// `avatar` should be the position of the player. If it is all zero,
        /// positional audio will be disabled. `camera` should be the position of
        /// the camera, which may be the same as `avatar`.
        $(#[$attr])*
        pub fn update(&mut $s, $a: Position, $c: Position) $b
    };
}

/// An active Mumble link connection.
pub struct MumbleLink {
    map: imp::Map,
    local: LinkedMem,
}

impl std::fmt::Debug for MumbleLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.local.fmt(f)
    }
}

impl MumbleLink {
    /// Attempt to open the Mumble link, providing the specified application
    /// name and description.
    ///
    /// Opening the link will fail if Mumble is not running. If another
    /// application is also using Mumble link, its data may be overwritten or
    /// conflict with this link. To avoid this, use `SharedLink`.
    pub fn new(name: &str, description: &str) -> Result<Self, ErrorCode> {
        Ok(Self {
            map: imp::Map::new(std::mem::size_of::<LinkedMem>())?,
            local: LinkedMem::new(name, description),
        })
    }

    docs! {
        #[inline]
        pub fn set_context(&mut self, context: &[u8]) {
            self.local.set_context(context)
        }
    }
    docs! {
        #[inline]
        pub fn set_identity(&mut self, identity: &str) {
            self.local.set_identity(identity)
        }
    }
    docs! {
        #[inline]
        pub fn update(&mut self, avatar: Position, camera: Position) {
            self.local.update(avatar, camera);
            unsafe {
                ptr::write_volatile(self.map.ptr as *mut LinkedMem, self.local);
            }
        }
    }
}

unsafe impl Send for MumbleLink {}

impl Drop for MumbleLink {
    fn drop(&mut self) {
        unsafe {
            // zero the linked memory
            ptr::write_volatile(self.map.ptr as *mut LinkedMem, mem::zeroed());
        }
    }
}

/// A weak Mumble link connection.
///
/// Constructing a `SharedLink` always succeeds, even if Mumble is not running
/// or another application is writing to the link. If this happens, `update()`
/// will retry opening the link on a regular basis, succeeding if Mumble is
/// started or the other application stops using the link.
pub struct SharedLink {
    inner: Inner,
    local: LinkedMem,
}

impl SharedLink {
    /// Open the Mumble link, providing the specified application name and
    /// description.
    pub fn new(name: &str, description: &str) -> SharedLink {
        SharedLink {
            inner: Inner::open(),
            local: LinkedMem::new(name, description),
        }
    }

    docs! {
        #[inline]
        pub fn set_context(&mut self, context: &[u8]) {
            self.local.set_context(context)
        }
    }

    docs! {
        #[inline]
        pub fn set_identity(&mut self, identity: &str) {
            self.local.set_identity(identity)
        }
    }

    docs! {
        pub fn update(&mut self, avatar: Position, camera: Position) {
            self.local.update(avatar, camera);

            // If it's been a hundred ticks, try to reopen the link
            if self.local.ui_tick % 100 == 0 {
                self.inner = match mem::replace(&mut self.inner, Inner::Unset) {
                    Inner::Closed(_) => Inner::open(),
                    Inner::InUse(map, last_tick) => {
                        let previous = unsafe { ptr::read_volatile(map.ptr as *mut LinkedMem) };
                        if previous.ui_version == 0 || last_tick == previous.ui_tick {
                            Inner::Active(map)
                        } else {
                            Inner::InUse(map, previous.ui_tick)
                        }
                    }
                    Inner::Active(map) => Inner::Active(map),
                    Inner::Unset => unreachable!(),
                };
            }

            // If the link is active, write to it
            if let Inner::Active(ref mut map) = self.inner {
                unsafe {
                    ptr::write_volatile(map.ptr as *mut LinkedMem, self.local);
                }
            }
        }
    }

    /// Get the status of the shared link. See `Status` for details.
    pub fn status(&self) -> Status {
        match self.inner {
            Inner::Closed(ref err) => Status::Closed(err),
            Inner::InUse(ref map, _) => {
                let previous = unsafe { ptr::read_volatile(map.ptr as *mut LinkedMem) };
                Status::InUse {
                    name: imp::read(&previous.name),
                    description: imp::read(&previous.description),
                }
            }
            Inner::Active(_) => Status::Active,
            Inner::Unset => unreachable!(),
        }
    }

    /// Deactivate the shared link.
    ///
    /// Should be called when `update()` will not be called again for a while,
    /// such as if the player is no longer in-game.
    pub fn deactivate(&mut self) {
        if let Inner::Active(ref mut map) = self.inner {
            unsafe {
                ptr::write_volatile(map.ptr as *mut LinkedMem, mem::zeroed());
            }
        }
        self.inner = Inner::Closed(io::Error::new(io::ErrorKind::Other, "Manually closed"));
    }
}

unsafe impl Send for SharedLink {}

impl Drop for SharedLink {
    fn drop(&mut self) {
        self.deactivate();
    }
}

enum Inner {
    Unset,
    Closed(io::Error),
    InUse(imp::Map, u32),
    Active(imp::Map),
}

impl Inner {
    fn open() -> Inner {
        match imp::Map::new(std::mem::size_of::<LinkedMem>()) {
            Err(_) => Inner::Closed(io::Error::last_os_error()),
            Ok(map) => {
                let previous = unsafe { ptr::read_volatile(map.ptr as *mut LinkedMem) };
                if previous.ui_version != 0 {
                    Inner::InUse(map, previous.ui_tick)
                } else {
                    Inner::Active(map)
                }
            }
        }
    }
}

/// The status of a `SharedLink`.
#[derive(Debug)]
pub enum Status<'a> {
    /// The link is closed. This is usually because Mumble is not running or
    /// the link was closed manually with `deactivate()`.
    Closed(&'a io::Error),
    /// The link is in use by another application.
    InUse {
        /// The name of the other application.
        name: String,
        /// The description of the other application.
        description: String,
    },
    /// The link is active.
    Active,
}

#[test]
fn test_wide() {
    let wide = wide!(M u m b l e L i n k);
    for (i, b) in "MumbleLink".bytes().enumerate() {
        assert_eq!(b as wchar_t, wide[i]);
    }
    assert_eq!(0, wide[wide.len() - 1]);

    let mut wide = [1; 32];
    imp::copy(&mut wide, "FooBar");
    assert_eq!(&wide[..7], wide!(F o o B a r));
    assert_eq!("FooBar", imp::read(&wide));

    let mut wide = [1; 3];
    imp::copy(&mut wide, "ABC");
    assert_eq!(&wide[..], wide!(A B));
    assert_eq!("AB", imp::read(&wide));

    assert_eq!("BarFoo", imp::read(&wide!(B a r F o o)));
}
