use_prelude!();

use ::core::mem::MaybeUninit;

pub
struct CellOption<T> /* = */ {
    is_some: Cell<bool>,
    value: Cell<MaybeUninit<T>>,
}

impl<T> CellOption<T> {
    #[allow(bad_style)]
    pub
    const None: Self = Self {
        is_some: Cell::new(false),
        value: Cell::new(MaybeUninit::uninit()),
    };

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

    pub
    fn take (self: &'_ Self)
      -> Option<T>
    {
        if self.is_some() {
            self.is_some.set(false);
            Some(unsafe {
                self.value
                    .replace(MaybeUninit::uninit())
                    .assume_init()
            })
        } else {
            None
        }
    }

    pub
    fn set (self: &'_ Self, value: T)
      -> Option<T>
    {
        let prev = self.value.replace(MaybeUninit::new(value));
        if self.is_some() {
            Some(unsafe { prev.assume_init() })
        } else {
            self.is_some.set(true);
            None
        }
    }

    #[inline]
    pub
    fn is_some (self: &'_ Self)
      -> bool
    {
        self.is_some.get()
    }
}
