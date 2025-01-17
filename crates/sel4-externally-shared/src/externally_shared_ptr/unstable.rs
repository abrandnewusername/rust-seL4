use core::{
    ops::{Range, RangeBounds},
    ptr::{self, NonNull},
    slice::{range, SliceIndex},
};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{
    access::{Access, Readable, Writable},
    ExternallySharedPtr,
};

impl<'a, T, A> ExternallySharedPtr<'a, [T], A> {
    /// Returns the length of the slice.
    pub fn len(self) -> usize {
        self.pointer.len()
    }

    /// Returns whether the slice is empty.
    pub fn is_empty(self) -> bool {
        self.pointer.len() == 0
    }

    /// Applies the index operation on the wrapped slice.
    ///
    /// Returns a shared `ExternallySharedPtr` reference to the resulting subslice.
    ///
    /// This is a convenience method for the `map(|slice| slice.index(index))` operation, so it
    /// has the same behavior as the indexing operation on slice (e.g. panic if index is
    /// out-of-bounds).
    ///
    /// ## Examples
    ///
    /// Accessing a single slice element:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let shared = unsafe { ExternallySharedPtr::new_read_only(NonNull::from(slice)) };
    /// assert_eq!(shared.index(1).read(), 2);
    /// ```
    ///
    /// Accessing a subslice:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let array = [1, 2, 3];
    /// let slice = &array[..];
    /// let shared = unsafe { ExternallySharedPtr::new_read_only(NonNull::from(slice)) };
    /// let subslice = shared.index(1..);
    /// assert_eq!(subslice.index(0).read(), 2);
    /// ```
    pub fn index<I>(self, index: I) -> ExternallySharedPtr<'a, <I as SliceIndex<[T]>>::Output, A>
    where
        I: SliceIndex<[T]> + SliceIndex<[()]> + Clone,
        A: Access,
    {
        bounds_check(self.pointer.len(), index.clone());

        unsafe { self.map(|slice| slice.get_unchecked_mut(index)) }
    }

    /// Returns an iterator over the slice.
    pub fn iter(self) -> impl Iterator<Item = ExternallySharedPtr<'a, T, A>>
    where
        A: Access,
    {
        let ptr = self.as_raw_ptr().as_ptr() as *mut T;
        let len = self.len();
        (0..len).map(move |i| unsafe {
            ExternallySharedPtr::new_generic(NonNull::new_unchecked(ptr.add(i)))
        })
    }

    /// Copies all elements from `self` into `dst`, using memcpy.
    ///
    /// The length of `dst` must be the same as `self`.
    ///
    /// The method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if the two slices have different lengths.
    ///
    /// ## Examples
    ///
    /// Copying two elements from a wrapped slice:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2];
    /// // the `ExternallySharedPtr` type does not work with arrays, so convert `src` to a slice
    /// let slice = &src[..];
    /// let shared = unsafe { ExternallySharedPtr::new_read_only(NonNull::from(slice)) };
    /// let mut dst = [5, 0, 0];
    ///
    /// // Because the slices have to be the same length,
    /// // we slice the destination slice from three elements
    /// // to two. It will panic if we don't do this.
    /// shared.copy_into_slice(&mut dst[1..]);
    ///
    /// assert_eq!(src, [1, 2]);
    /// assert_eq!(dst, [5, 1, 2]);
    /// ```
    pub fn copy_into_slice(self, dst: &mut [T])
    where
        T: Copy,
        A: Readable,
    {
        let len = self.pointer.len();
        assert_eq!(
            len,
            dst.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            dst.as_mut_ptr()
                .copy_from_nonoverlapping(self.pointer.as_mut_ptr(), len);
        }
    }

    /// Copies all elements from `src` into `self`, using memcpy.
    ///
    /// The length of `src` must be the same as `self`.
    ///
    /// The method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if the two slices have different lengths.
    ///
    /// ## Examples
    ///
    /// Copying two elements from a slice into a wrapped slice:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2, 3, 4];
    /// let mut dst = [0, 0];
    /// // the `ExternallySharedPtr` type does not work with arrays, so convert `dst` to a slice
    /// let slice = &mut dst[..];
    /// let mut shared = unsafe { ExternallySharedPtr::new(NonNull::from(slice)) };
    /// // Because the slices have to be the same length,
    /// // we slice the source slice from four elements
    /// // to two. It will panic if we don't do this.
    /// shared.copy_from_slice(&src[2..]);
    ///
    /// assert_eq!(src, [1, 2, 3, 4]);
    /// assert_eq!(dst, [3, 4]);
    /// ```
    pub fn copy_from_slice(self, src: &[T])
    where
        T: Copy,
        A: Writable,
    {
        let len = self.pointer.len();
        assert_eq!(
            len,
            src.len(),
            "destination and source slices have different lengths"
        );
        unsafe {
            self.pointer
                .as_mut_ptr()
                .copy_from_nonoverlapping(src.as_ptr(), len);
        }
    }

    /// Copies elements from one part of the slice to another part of itself, using `memmove`.
    ///
    /// `src` is the range within `self` to copy from. `dest` is the starting index of the
    /// range within `self` to copy to, which will have the same length as `src`. The two ranges
    /// may overlap. The ends of the two ranges must be less than or equal to `self.len()`.
    ///
    /// This method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Panics
    ///
    /// This function will panic if either range exceeds the end of the slice, or if the end
    /// of `src` is before the start.
    ///
    /// ## Examples
    ///
    /// Copying four bytes within a slice:
    ///
    /// ```
    /// extern crate core;
    /// use sel4_externally_shared::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let mut byte_array = *b"Hello, World!";
    /// let mut slice: &mut [u8] = &mut byte_array[..];
    /// let mut shared = unsafe { ExternallySharedPtr::new(NonNull::from(slice)) };
    /// shared.copy_within(1..5, 8);
    ///
    /// assert_eq!(&byte_array, b"Hello, Wello!");
    pub fn copy_within(self, src: impl RangeBounds<usize>, dest: usize)
    where
        T: Copy,
        A: Readable + Writable,
    {
        let len = self.pointer.len();
        // implementation taken from https://github.com/rust-lang/rust/blob/683d1bcd405727fcc9209f64845bd3b9104878b8/library/core/src/slice/mod.rs#L2726-L2738
        let Range {
            start: src_start,
            end: src_end,
        } = range(src, ..len);
        let count = src_end - src_start;
        assert!(dest <= len - count, "dest is out of bounds");
        unsafe {
            self.pointer
                .as_mut_ptr()
                .add(dest)
                .copy_from(self.pointer.as_mut_ptr().add(src_start), count);
        }
    }

    /// Divides one slice into two at an index.
    ///
    /// The first will contain all indices from `[0, mid)` (excluding
    /// the index `mid` itself) and the second will contain all
    /// indices from `[mid, len)` (excluding the index `len` itself).
    ///
    /// # Panics
    ///
    /// Panics if `mid > len`.
    ///
    pub fn split_at(
        self,
        mid: usize,
    ) -> (
        ExternallySharedPtr<'a, [T], A>,
        ExternallySharedPtr<'a, [T], A>,
    )
    where
        A: Access,
    {
        assert!(mid <= self.pointer.len());
        // SAFETY: `[ptr; mid]` and `[mid; len]` are inside `self`, which
        // fulfills the requirements of `from_raw_parts_mut`.
        unsafe { self.split_at_unchecked(mid) }
    }

    unsafe fn split_at_unchecked(
        self,
        mid: usize,
    ) -> (
        ExternallySharedPtr<'a, [T], A>,
        ExternallySharedPtr<'a, [T], A>,
    )
    where
        A: Access,
    {
        // SAFETY: Caller has to check that `0 <= mid <= self.len()`
        unsafe {
            (
                ExternallySharedPtr::new_generic((self.pointer).get_unchecked_mut(..mid)),
                ExternallySharedPtr::new_generic((self.pointer).get_unchecked_mut(mid..)),
            )
        }
    }

    /// Splits the slice into a slice of `N`-element arrays,
    /// starting at the beginning of the slice,
    /// and a remainder slice with length strictly less than `N`.
    ///
    /// # Panics
    ///
    /// Panics if `N` is 0.
    #[allow(clippy::type_complexity)]
    pub fn as_chunks<const N: usize>(
        self,
    ) -> (
        ExternallySharedPtr<'a, [[T; N]], A>,
        ExternallySharedPtr<'a, [T], A>,
    )
    where
        A: Access,
    {
        assert_ne!(N, 0);
        let len = self.pointer.len() / N;
        let (multiple_of_n, remainder) = self.split_at(len * N);
        // SAFETY: We already panicked for zero, and ensured by construction
        // that the length of the subslice is a multiple of N.
        let array_slice = unsafe { multiple_of_n.as_chunks_unchecked() };
        (array_slice, remainder)
    }

    /// Splits the slice into a slice of `N`-element arrays,
    /// assuming that there's no remainder.
    ///
    /// # Safety
    ///
    /// This may only be called when
    /// - The slice splits exactly into `N`-element chunks (aka `self.len() % N == 0`).
    /// - `N != 0`.
    pub unsafe fn as_chunks_unchecked<const N: usize>(self) -> ExternallySharedPtr<'a, [[T; N]], A>
    where
        A: Access,
    {
        debug_assert_ne!(N, 0);
        debug_assert_eq!(self.pointer.len() % N, 0);
        let new_len =
            // SAFETY: Our precondition is exactly what's needed to call this
            unsafe { core::intrinsics::exact_div(self.pointer.len(), N) };
        // SAFETY: We cast a slice of `new_len * N` elements into
        // a slice of `new_len` many `N` elements chunks.
        let pointer = NonNull::new(ptr::slice_from_raw_parts_mut(
            self.pointer.as_mut_ptr().cast(),
            new_len,
        ))
        .unwrap();
        unsafe { ExternallySharedPtr::new_generic(pointer) }
    }

    /// Copies all elements from `self` into a `Vec`.
    #[cfg(feature = "alloc")]
    pub fn copy_to_vec(&self) -> Vec<T>
    where
        T: Copy,
    {
        let src = self.pointer.as_mut_ptr();
        let n = self.pointer.len();
        let mut v = Vec::with_capacity(n);
        // SAFETY:
        // allocated above with the capacity of `src`, and initialize to `src.len()` in
        // ptr::copy_to_non_overlapping below.
        unsafe {
            src.copy_to_nonoverlapping(v.as_mut_ptr(), n);
            v.set_len(n);
        }
        v
    }
}

/// Methods for wrapped byte slices
impl<A> ExternallySharedPtr<'_, [u8], A> {
    /// Sets all elements of the byte slice to the given `value` using `memset`.
    ///
    /// This method is only available with the `unstable` feature enabled (requires a nightly
    /// Rust compiler).
    ///
    /// ## Example
    ///
    /// ```rust
    /// use sel4_externally_shared::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let mut vec = vec![0; 10];
    /// let mut buf = unsafe { ExternallySharedPtr::new(NonNull::from(vec.as_mut_slice())) };
    /// buf.fill(1);
    /// assert_eq!(unsafe { buf.as_raw_ptr().as_mut() }, &mut vec![1; 10]);
    /// ```
    pub fn fill(self, value: u8)
    where
        A: Writable,
    {
        unsafe {
            self.pointer
                .as_mut_ptr()
                .write_bytes(value, self.pointer.len());
        }
    }
}

/// Methods for converting arrays to slices
///
/// These methods are only available with the `unstable` feature enabled (requires a nightly
/// Rust compiler).
impl<'a, T, A, const N: usize> ExternallySharedPtr<'a, [T; N], A> {
    /// Converts an array pointer to a slice pointer.
    ///
    /// This makes it possible to use the methods defined on slices.
    ///
    /// ## Example
    ///
    /// Copying two elements from an array reference using `copy_into_slice`:
    ///
    /// ```
    /// use sel4_externally_shared::ExternallySharedPtr;
    /// use core::ptr::NonNull;
    ///
    /// let src = [1, 2];
    /// let shared = unsafe { ExternallySharedPtr::new_read_only(NonNull::from(&src)) };
    /// let mut dst = [0, 0];
    ///
    /// // convert the `ExternallySharedPtr<&[i32; 2]>` array reference to a `ExternallySharedPtr<&[i32]>` slice
    /// let shared_slice = shared.as_slice();
    /// // we can now use the slice methods
    /// shared_slice.copy_into_slice(&mut dst);
    ///
    /// assert_eq!(dst, [1, 2]);
    /// ```
    pub fn as_slice(self) -> ExternallySharedPtr<'a, [T], A>
    where
        A: Access,
    {
        unsafe {
            self.map(|array| {
                NonNull::new(ptr::slice_from_raw_parts_mut(array.as_ptr() as *mut T, N)).unwrap()
            })
        }
    }
}

fn bounds_check(len: usize, index: impl SliceIndex<[()]>) {
    const MAX_ARRAY: [(); usize::MAX] = [(); usize::MAX];

    let bound_check_slice = &MAX_ARRAY[..len];
    let _ = &bound_check_slice[index];
}
