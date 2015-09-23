/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::{JSTracer, AutoGCRooter, RootedGeneric, AutoGCRooter_};
use jsapi::{JSContext, ContextFriendFields, _vftable_CustomAutoRooter};
use jsapi::CustomAutoRooter as JSCustomAutoRooter;
use libc;
use std::intrinsics::return_address;
use std::ops::{Deref, DerefMut};

#[macro_export]
macro_rules! make_vtable(
    ($name:ident, $t:ty) => (
        static $name: _vftable_CustomAutoRooter {
            trace: trace::<$t>,
        };
    )
);

pub trait CustomTraceable {
    fn trace_fields(&self, trc: *mut JSTracer);
    fn vtable() -> &'static _vftable_CustomAutoRooter;
}

extern "C" fn trace<T: CustomTraceable>(this: *mut libc::c_void, trc: *mut JSTracer) {
    unsafe {
        let this = this as *mut T;
        (*this).trace_fields(trc);
    }
}

pub struct CustomAutoRooter<T: CustomTraceable> {
    inner: RootedGeneric<T>
}

impl<T: CustomTraceable> CustomAutoRooter<T> {
    pub fn new(cx: *mut JSContext, val: T) -> CustomAutoRooter<T> {
        unsafe {
            let cx = cx as *mut ContextFriendFields;
            let this = return_address() as *mut AutoGCRooter;

            let result = CustomAutoRooter {
                inner: RootedGeneric {
                    _base: JSCustomAutoRooter {
                        _vftable: T::vtable(),
                        _base: AutoGCRooter {
                            down: (*cx).autoGCRooters,
                            tag_: AutoGCRooter_::CUSTOM as i32,
                            stackTop: &mut (*cx).autoGCRooters,
                        },
                    },
                    value: val,
                },
            };

            assert!(this != *result.inner._base._base.stackTop);
            (*cx).autoGCRooters = this;


            result
        }
    }
}

impl<T: CustomTraceable> Drop for CustomAutoRooter<T> {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(&mut self.inner._base._base as *mut AutoGCRooter,
                       *self.inner._base._base.stackTop as *mut AutoGCRooter);
            *self.inner._base._base.stackTop = self.inner._base._base.down;
        }
    }
}

impl<T: CustomTraceable> Deref for CustomAutoRooter<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.inner.value
    }
}

impl<T: CustomTraceable> DerefMut for CustomAutoRooter<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner.value
    }
}
