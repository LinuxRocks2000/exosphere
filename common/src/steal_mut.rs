/*
    you know what? skip the copyright on this one. let's just pretend it never happened.
*/

// evade memory protections. turn an &-reference into an &mut-reference naively, without any regard for human life.


struct InvarianceBreaker<T> {
    invariant : T
}


impl<T> InvarianceBreaker<T> { // what is this?
    // rust is VERY careful about preventing you from casting immutable to mutable, and the static analyzer is smart.
    // you can transmute the fuckin' *const pointer to usize and then transmute the usize to *mut, and it will *still* catch that
    // as being a cast from & to &mut. so there's... this.
    // this class ensures that your usize from the first `transmute` is moved out of the original function scope into our purdy little InvarianceBreaker,
    // and then the InvarianceBreaker is consumed to return your usize (or whatever). because it's moved around in memory multiple times, that little
    // usize is now impossible to identify (by the static analyzer, anyways) as being originally a *const pointer, so you can cast it to *mut
    unsafe fn new(d : T) -> Self {
        Self {
            invariant : d
        }
    }

    unsafe fn shatter(self) -> T {
        self.invariant
    }
}


pub fn steal_mut<T>(t : &T) -> &mut T {
    // you probably read the function signature and immediately went "uh-oh".
    // yeah, it ain't good
    // this is an unsafe (and private) helper to cast an immutable reference to... a mutable one.
    // the reason for this to exist is that all of the operations on PathFollower are single-threaded and mut-safe
    // - a mutable reference and an immutable reference to data inside a PathFollower are never held.
    // However, there are cases where a mutable reference to a structure containing a PathFollower is held while
    // an immutable reference to that structure is taken for a function, which is not possible under Rust mutability rules (even though
    // the pathfollower isn't touched inside that function)
    // RefCell et al are heavy and not particularly ergonomic. So this function exists! cheaply evade
    // static analysis at the cost of safety.
    // this better not become a habit.

    // update 2025-4-30: I have no idea what the fuck I was talking about here.
    //                   RefCell and other interior mutability systems are cheap enough that their overhead will never matter.
    //                   This function will be kept for historical value, but of high priority is removing every use from the code.
    return unsafe {
        let raw = t as *const T;
        let even_rawer = InvarianceBreaker::new(std::mem::transmute::<*const T, usize>(raw));
        &mut *(std::mem::transmute::<usize, *mut T>(even_rawer.shatter()))
    }
}
