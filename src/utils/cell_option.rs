use_prelude!();

use ::core::mem::MaybeUninit;

/// Like `Cell<Option<T>>`, but ensures no layout optimization takes place so
/// that the `Some/None` discriminant can be read directly.
//
// # Safety invariant
//
//   - if `is_some`, then `value` is sound to `assume_init()`
pub
struct CellOption<T> /* = */ {
    is_some: Cell<bool>,
    value: Cell<MaybeUninit<T>>,
}

impl<T> Default for CellOption<T> {
    #[inline]
    fn default ()
      -> Self
    {
        Self::None
    }
}

impl<T> CellOption<T> {
    /// `Cell::new(None)`
    #[allow(bad_style)]
    pub
    const None: Self = Self {
        is_some: Cell::new(false),
        value: Cell::new(MaybeUninit::uninit()),
    };

    /// `Cell::new(Some(value))`
    #[cfg(FALSE)]
    #[allow(bad_style)]
    pub
    const
    fn Some (value: T)
      -> Self
    {
        Self {
            value: Cell::new(MaybeUninit::new(value)),
            is_some: Cell::new(true),
        }
    }

    /// `Cell::replace(self, None)`
    pub
    fn take (self: &'_ Self)
      -> Option<T>
    {
        if self.is_some() { // ------------------------+
            self.is_some.set(false);                // | # Safety
            Some(unsafe {                           // |
                self.value                          // |   - value was init
                    .replace(MaybeUninit::uninit()) // |
                    .assume_init() // <----------------+
            })
        } else {
            None
        }
    }

    /// `Cell::replace(self, Some(value))`
    pub
    fn set (self: &'_ Self, value: T)
      -> Option<T>
    {
        let prev = self.value.replace(MaybeUninit::new(value));
        if self.is_some() {
            Some(unsafe {
                // # Safety
                //
                // From safety invariant.
                prev.assume_init()
            })
        } else {
            self.is_some.set(true);
            None
        }
    }

    /// Returns `true` if and only if the `Cell` contains a value.
    #[inline]
    pub
    fn is_some (self: &'_ Self)
      -> bool
    {
        self.is_some.get()
    }
}
