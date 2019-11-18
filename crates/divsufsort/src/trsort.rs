use crate::{common::*, crosscheck, crosscheck::*, SA_dump};
use std::mem;

//--------------------
// Private functions
//--------------------

#[rustfmt::skip]
const lg_table: [Idx; 256] = [
 -1,0,1,1,2,2,2,2,3,3,3,3,3,3,3,3,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,
  5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,
  6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,
  6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,6,
  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,
  7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7,7
];

#[inline(always)]
#[allow(overflowing_literals)]
pub fn tr_ilg<N: Into<Idx>>(n: N) -> Idx {
    let n = n.into();
    if (n & 0xffff_0000) > 0 {
        if (n & 0xff00_0000) > 0 {
            24 + lg_table[((n >> 24) & 0xff) as usize]
        } else {
            16 + lg_table[((n >> 16) & 0xff) as usize]
        }
    } else {
        if (n & 0x0000_ff00) > 0 {
            8 + lg_table[((n >> 8) & 0xff) as usize]
        } else {
            0 + lg_table[((n >> 0) & 0xff) as usize]
        }
    }
}

//------------------------------------------------------------------------------

use std::default::Default;
const STACK_SIZE: usize = 64;

#[derive(Default)]
struct StackItem {
    a: SAPtr,
    b: SAPtr,
    c: SAPtr,
    d: Idx,
    e: Idx,
}

struct Stack {
    items: smallvec::SmallVec<[StackItem; STACK_SIZE]>,
}

impl Stack {
    fn new() -> Self {
        Self {
            items: smallvec::SmallVec::new(),
        }
    }

    #[inline(always)]
    fn size(&self) -> usize {
        self.items.len()
    }

    #[inline(always)]
    fn push(&mut self, a: SAPtr, b: SAPtr, c: SAPtr, d: Idx, e: Idx) {
        self.items.push(StackItem{ a, b, c, d, e });
    }

    #[inline(always)]
    #[must_use]
    fn pop(
        &mut self,
        a: &mut SAPtr,
        b: &mut SAPtr,
        c: &mut SAPtr,
        d: &mut Idx,
        e: &mut Idx,
    ) -> Result<(), ()> {
        if let Some(i) = self.items.pop() {
            *a = i.a;
            *b = i.b;
            *c = i.c;
            *d = i.d;
            *e = i.e;
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline(always)]
    fn pop_unused(&mut self) {
        self.items.pop().unwrap();
    }
}

//------------------------------------------------------------------------------

/// Simple insertionsort for small size groups
pub fn tr_insertionsort(SA: &mut SuffixArray, ISAd: SAPtr, first: SAPtr, last: SAPtr) {
    let mut a: SAPtr;
    let mut b: SAPtr;
    let mut t: Idx;
    let mut r: Idx;

    macro_rules! ISAd {
        ($x: expr) => {
            SA[ISAd + $x]
        };
    }

    a = first + 1;
    // KAREN
    while a < last {
        // JEZEBEL
        t = SA[a];
        b = a - 1;
        loop {
            // cond (JEZEBEL)
            r = ISAd!(t) - ISAd!(SA[b]);
            if !(0 > r) {
                break;
            }

            // LILITH
            loop {
                SA[b + 1] = SA[b];

                // cond (LILITH)
                b -= 1;
                if !((first <= b) && (SA[b] < 0)) {
                    break;
                }
            }

            // body (JEZEBEL)
            if b < first {
                break;
            }
        }

        if r == 0 {
            SA[b] = !SA[b];
        }
        SA[b + 1] = t;

        // iter
        a += 1;
    }
}

//------------------------------------------------------------------------------

#[inline(always)]
pub fn tr_fixdown(ISAd: SAPtr, SA_top: &mut SuffixArray, first: SAPtr, mut i: Idx, size: Idx) {
    let mut j: Idx;
    let mut k: Idx;
    let mut d: Idx;
    let mut e: Idx;

    crosscheck!("fixdown i={} size={}", i, size);

    macro_rules! ISAd {
        ($x: expr) => {
            SA_top[ISAd + $x]
        };
    }
    macro_rules! SA {
        ($x: expr) => {
            SA_top[first + $x]
        };
    };

    // WILMOT
    let v = SA!(i);
    let c = ISAd!(v);
    loop {
        // cond
        j = 2 * i + 1;
        if !(j < size) {
            break;
        }

        // body
        k = j;
        d = ISAd!(SA!(k));
        j += 1;
        e = ISAd!(SA!(j));
        if d < e {
            k = j;
            d = e;
        }
        if d <= c {
            break;
        }

        // iter (WILMOT)
        SA!(i) = SA!(k);
        i = k;
    }
    SA!(i) = v;
}

/// Simple top-down heapsort
pub fn tr_heapsort(ISAd: SAPtr, SA_top: &mut SuffixArray, first: SAPtr, size: Idx) {
    let mut i: Idx;
    let mut m: Idx;
    let mut t: Idx;

    macro_rules! ISAd {
        ($x: expr) => {
            SA_top[ISAd + $x]
        };
    }
    macro_rules! SA {
        ($x: expr) => {
            SA_top[first + $x]
        };
    }
    macro_rules! SA_swap {
        ($a: expr, $b: expr) => {
            SA_top.swap(first + $a, first + $b);
        };
    }

    m = size;
    if (size % 2) == 0 {
        m -= 1;
        if ISAd!(SA!(m / 2)) < ISAd!(SA!(m)) {
            SA_swap!(m, (m / 2));
        }
    }

    // LISA
    for i in (0..(m / 2)).rev() {
        crosscheck!("LISA i={}", i);
        tr_fixdown(ISAd, SA_top, first, i, m);
    }
    if (size % 2) == 0 {
        SA_swap!(0, m);
        tr_fixdown(ISAd, SA_top, first, 0, m);
    }
    // MARK
    for i in (1..m).rev() {
        crosscheck!("MARK i={}", i);
        t = SA!(0);
        SA!(0) = SA!(i);
        tr_fixdown(ISAd, SA_top, first, 0, i);
        SA!(i) = t;
    }
}

//------------------------------------------------------------------------------

/// Returns the median of three elements
#[inline(always)]
pub fn tr_median3(SA: &SuffixArray, ISAd: SAPtr, mut v1: SAPtr, mut v2: SAPtr, v3: SAPtr) -> SAPtr {
    macro_rules! get {
        ($x: expr) => {
            SA[ISAd + SA[$x]]
        };
    }

    if get!(v1) > get!(v2) {
        mem::swap(&mut v1, &mut v2);
    }
    if get!(v2) > get!(v3) {
        if get!(v1) > get!(v3) {
            v1
        } else {
            v3
        }
    } else {
        v2
    }
}

/// Returns the median of five elements
#[inline(always)]
pub fn tr_median5(
    SA: &SuffixArray,
    ISAd: SAPtr,
    mut v1: SAPtr,
    mut v2: SAPtr,
    mut v3: SAPtr,
    mut v4: SAPtr,
    mut v5: SAPtr,
) -> SAPtr {
    macro_rules! get {
        ($x: expr) => {
            SA[ISAd + SA[$x]]
        };
    }
    if get!(v2) > get!(v3) {
        mem::swap(&mut v2, &mut v3);
    }
    if get!(v4) > get!(v5) {
        mem::swap(&mut v4, &mut v5);
    }
    if get!(v2) > get!(v4) {
        mem::swap(&mut v2, &mut v4);
        mem::swap(&mut v3, &mut v5);
    }
    if get!(v1) > get!(v3) {
        mem::swap(&mut v1, &mut v3);
    }
    if get!(v1) > get!(v4) {
        mem::swap(&mut v1, &mut v4);
        mem::swap(&mut v3, &mut v5);
    }
    if get!(v3) > get!(v4) {
        v4
    } else {
        v3
    }
}

/// Returns the pivot element
#[inline(always)]
pub fn tr_pivot(SA: &SuffixArray, ISAd: SAPtr, mut first: SAPtr, mut last: SAPtr) -> SAPtr {
    let mut t: Idx = (last - first).0;
    let mut middle: SAPtr = first + t / 2;

    if t <= 512 {
        if t <= 32 {
            return tr_median3(SA, ISAd, first, middle, last - 1);
        } else {
            t >>= 2;
            return tr_median5(SA, ISAd, first, first + t, middle, last - 1 - t, last - 1);
        }
    }
    t >>= 3;
    first = tr_median3(SA, ISAd, first, first + t, first + (t << 1));
    middle = tr_median3(SA, ISAd, middle - t, middle, middle + t);
    last = tr_median3(SA, ISAd, last - 1 - (t << 1), last - 1 - t, last - 1);
    tr_median3(SA, ISAd, first, middle, last)
}

//------------------------------------------------------------------------------

pub struct Budget {
    pub chance: Idx,
    pub remain: Idx,
    pub incval: Idx,
    pub count: Idx,
}

impl Budget {
    pub fn new(chance: Idx, incval: Idx) -> Self {
        Self {
            chance,
            remain: incval,
            incval,
            count: 0,
        }
    }

    pub fn check<S: Into<Idx>>(&mut self, size: S) -> bool {
        let size = size.into();
        if (size <= self.remain) {
            self.remain -= size;
            return true;
        }

        if (self.chance == 0) {
            self.count += size;
            return false;
        }

        self.remain += self.incval - size;
        self.chance -= 1;
        return true;
    }
}

//------------------------------------------------------------------------------

/// Tandem repeat partition
#[inline(always)]
pub fn tr_partition(
    SA: &mut SuffixArray,
    ISAd: SAPtr,
    mut first: SAPtr,
    middle: SAPtr,
    mut last: SAPtr,
    pa: &mut SAPtr,
    pb: &mut SAPtr,
    v: Idx,
) {
    let mut a: SAPtr;
    let mut b: SAPtr;
    let mut c: SAPtr;
    let mut d: SAPtr;
    let mut e: SAPtr;
    let mut f: SAPtr;
    let mut t: Idx;
    let mut s: Idx;
    let mut x: Idx = 0;

    macro_rules! get {
        ($x: expr) => {
            SA[ISAd + SA[$x]]
        };
    }

    // JOSEPH
    b = middle - 1;
    loop {
        // cond
        b += 1;
        if !(b < last) {
            break;
        }
        x = get!(b);
        if !(x == v) {
            break;
        }
    }
    a = b;
    if (a < last) && (x < v) {
        // MARY
        loop {
            b += 1;
            if !(b < last) {
                break;
            }
            x = get!(b);
            if !(x <= v) {
                break;
            }

            // body
            if (x == v) {
                SA.swap(b, a);
                a += 1;
            }
        }
    }

    // JEREMIAH
    c = last;
    loop {
        c -= 1;
        if !(b < c) {
            break;
        }
        x = get!(c);
        if !(x == v) {
            break;
        }
    }
    d = c;
    if (b < d) && (x > v) {
        // BEDELIA
        loop {
            c -= 1;
            if !(b < c) {
                break;
            }
            x = get!(c);
            if !(x >= v) {
                break;
            }
            if x == v {
                SA.swap(c, d);
                d -= 1;
            }
        }
    }

    // ALEX
    while b < c {
        SA.swap(b, c);
        // SIMON
        loop {
            b += 1;
            if !(b < c) {
                break;
            }
            x = get!(b);
            if !(x <= v) {
                break;
            }
            if x == v {
                SA.swap(b, a);
                a += 1;
            }
        }

        // GREGORY
        loop {
            c -= 1;
            if !(b < c) {
                break;
            }
            x = get!(c);
            if !(x >= v) {
                break;
            }
            if x == v {
                SA.swap(c, d);
                d -= 1;
            }
        }
    } // end ALEX

    if a <= d {
        c = b - 1;

        s = (a - first).0;
        t = (b - a).0;
        if (s > t) {
            s = t
        }

        // GENEVIEVE
        e = first;
        f = b - s;
        while 0 < s {
            SA.swap(e, f);
            s -= 1;
            e += 1;
            f += 1;
        }
        s = (d - c).0;
        t = (last - d - 1).0;
        if s > t {
            s = t;
        }

        // MARISSA
        e = b;
        f = last - s;
        while 0 < s {
            SA.swap(e, f);
            s -= 1;
            e += 1;
            f += 1;
        }
        first += (b - a);
        last -= (d - c).0;
    }
    pa.0 = first.0;
    pb.0 = last.0;
}

/// Tandem repeat copy
pub fn tr_copy(
    ISA: SAPtr,
    SA: &mut SuffixArray,
    first: SAPtr,
    a: SAPtr,
    b: SAPtr,
    last: SAPtr,
    depth: Idx,
) {
    // sort suffixes of middle partition
    // by using sorted order of suffixes of left and right partition.
    let mut c: SAPtr;
    let mut d: SAPtr;
    let mut e: SAPtr;
    let mut s: Idx;
    let mut v: Idx;

    crosscheck!("tr_copy first={} a={} b={} last={}", first, a, b, last);

    v = (b - 1).0;

    macro_rules! ISA {
        ($x: expr) => {
            SA[ISA + $x]
        };
    }

    // JACK
    c = first;
    d = a - 1;
    while c <= d {
        s = SA[c] - depth;
        if (0 <= s) && (ISA!(s) == v) {
            d += 1;
            SA[d] = s;
            ISA!(s) = d.0;
        }

        // iter (JACK)
        c += 1;
    }

    // JILL
    c = last - 1;
    e = d + 1;
    d = b;
    while e < d {
        s = SA[c] - depth;
        if (0 <= s) && (ISA!(s) == v) {
            d -= 1;
            SA[d] = s;
            ISA!(s) = d.0;
        }

        // iter (JILL)
        c -= 1;
    }
}

pub fn tr_partialcopy(
    ISA: SAPtr,
    SA: &mut SuffixArray,
    first: SAPtr,
    a: SAPtr,
    b: SAPtr,
    last: SAPtr,
    depth: Idx,
) {
    let mut c: SAPtr;
    let mut d: SAPtr;
    let mut e: SAPtr;
    let mut s: Idx;
    let mut v: Idx;
    let mut rank: Idx;
    let mut lastrank: Idx;
    let mut newrank: Idx = -1;

    macro_rules! ISA {
        ($x: expr) => {
            SA[ISA + $x]
        };
    }

    v = (b - 1).0;
    lastrank = -1;
    // JETHRO
    c = first;
    d = a - 1;
    while c <= d {
        s = SA[c] - depth;
        if (0 <= s) && (ISA!(s) == v) {
            d += 1;
            SA[d] = s;
            rank = ISA!(s + depth);
            if lastrank != rank {
                lastrank = rank;
                newrank = d.0;
            }
            ISA!(s) = newrank;
        }

        // iter (JETHRO)
        c += 1;
    }

    lastrank = -1;
    // SCROOGE
    e = d;
    while first <= e {
        rank = ISA![SA[e]];
        if lastrank != rank {
            lastrank = rank;
            newrank = e.0;
        }
        if newrank != rank {
            {
                let SA_e = SA[e];
                ISA!(SA_e) = newrank;
            }
        }

        // iter (SCROOGE)
        e -= 1;
    }

    lastrank = -1;
    // DEWEY
    c = last - 1;
    e = d + 1;
    d = b;
    while e < d {
        s = SA[c] - depth;
        if (0 <= s) && (ISA!(s) == v) {
            d -= 1;
            SA[d] = s;
            rank = ISA!(s + depth);
            if lastrank != rank {
                lastrank = rank;
                newrank = d.0;
            }
            ISA!(s) = newrank;
        }

        // iter (DEWEY)
        c -= 1;
    }
}

pub fn tr_introsort(
    ISA: SAPtr,
    mut ISAd: SAPtr,
    SA: &mut SuffixArray,
    mut first: SAPtr,
    mut last: SAPtr,
    budget: &mut Budget,
) {
    let mut a: SAPtr = SAPtr(0);
    let mut b: SAPtr = SAPtr(0);
    let mut c: SAPtr;
    let mut t: Idx;
    let mut v: Idx;
    let mut x: Idx;
    let mut incr: Idx = (ISAd - ISA).0;
    let mut limit: Idx;
    let mut next: Idx;
    let mut trlink: Idx = -1;

    let mut stack = Stack::new();

    macro_rules! ISA {
        ($x: expr) => {
            SA[ISA + $x]
        };
    }
    macro_rules! ISAd {
        ($x: expr) => {
            SA[ISAd + $x]
        };
    }

    let mut limit = tr_ilg(last - first);
    // PASCAL
    loop {
        crosscheck!("pascal limit={} first={} last={}", limit, first, last);
        if (limit < 0) {
            if (limit == -1) {
                // tandem repeat partition
                tr_partition(
                    SA,
                    ISAd - incr,
                    first,
                    first,
                    last,
                    &mut a,
                    &mut b,
                    (last - 1).0,
                );

                // update ranks
                if a < last {
                    crosscheck!("ranks a<last");
                    // JONAS
                    c = first;
                    v = (a - 1).0;
                    while c < a {
                        {
                            let SA_c = SA[c];
                            ISA!(SA_c) = v;
                        }

                        // iter (JONAS)
                        c += 1;
                    }
                }
                if b < last {
                    crosscheck!("ranks b<last");
                    // AHAB
                    c = a;
                    v = (b - 1).0;
                    while c < b {
                        {
                            let SA_c = SA[c];
                            ISA!(SA_c) = v;
                        }

                        // iter (AHAB)
                        c += 1;
                    }
                }

                // push
                if 1 < (b - a) {
                    crosscheck!("1<(b-a)");
                    crosscheck!("push NULL {} {} {} {}", a, b, 0, 0);
                    stack.push(SAPtr(0), a, b, 0, 0);
                    crosscheck!("push {} {} {} {} {}", ISAd - incr, first, last, -2, trlink);
                    stack.push(ISAd - incr, first, last, -2, trlink);
                    trlink = (stack.items.len() as Idx) - 2;
                }

                if (a - first) <= (last - b) {
                    crosscheck!("star");
                    if 1 < (a - first) {
                        crosscheck!("board");
                        crosscheck!(
                            "push {} {} {} {} {}",
                            ISAd,
                            b,
                            last,
                            tr_ilg(last - b),
                            trlink
                        );
                        stack.push(ISAd, b, last, tr_ilg(last - b), trlink);
                        last = a;
                        limit = tr_ilg(a - first);
                    } else if 1 < (last - b) {
                        crosscheck!("north");
                        first = b;
                        limit = tr_ilg(last - b);
                    } else {
                        crosscheck!("denny");
                        if !stack
                            .pop(&mut ISAd, &mut first, &mut last, &mut limit, &mut trlink)
                            .is_ok()
                        {
                            return;
                        }
                        crosscheck!("denny-post");
                    }
                } else {
                    crosscheck!("moon");
                    if 1 < (last - b) {
                        crosscheck!("land");
                        crosscheck!(
                            "push {} {} {} {} {}",
                            ISAd,
                            first,
                            a,
                            tr_ilg(a - first),
                            trlink
                        );
                        stack.push(ISAd, first, a, tr_ilg(a - first), trlink);
                        first = b;
                        limit = tr_ilg(last - b);
                    } else if 1 < (a - first) {
                        crosscheck!("ship");
                        last = a;
                        limit = tr_ilg(a - first);
                    } else {
                        crosscheck!("clap");
                        if !stack
                            .pop(&mut ISAd, &mut first, &mut last, &mut limit, &mut trlink)
                            .is_ok()
                        {
                            return;
                        }
                        crosscheck!("clap-post");
                    }
                }
            } else if (limit == -2) {
                // end if limit == -1

                // tandem repeat copy
                stack.pop_unused();
                a = stack.items[stack.size()].b;
                b = stack.items[stack.size()].c;

                if stack.items[stack.size()].d == 0 {
                    tr_copy(ISA, SA, first, a, b, last, (ISAd - ISA).0);
                } else {
                    if 0 <= trlink {
                        stack.items[trlink as usize].d = -1;
                    }
                    tr_partialcopy(ISA, SA, first, a, b, last, (ISAd - ISA).0);
                }
                if !stack
                    .pop(&mut ISAd, &mut first, &mut last, &mut limit, &mut trlink)
                    .is_ok()
                {
                    return;
                }
            } else {
                // end if limit == -2

                // sorted partition
                if 0 <= SA[first] {
                    crosscheck!("0<=*first");
                    a = first;
                    // GEMINI
                    loop {
                        {
                            let SA_a = SA[a];
                            ISA!(SA_a) = a.0;
                        }

                        // cond (GEMINI)
                        a += 1;
                        if !((a < last) && (0 <= SA[a])) {
                            break;
                        }
                    }
                    first = a;
                }

                if first < last {
                    crosscheck!("first<last");
                    a = first;
                    // MONSTRO
                    loop {
                        SA[a] = !SA[a];

                        a += 1;
                        if !(SA[a] < 0) {
                            break;
                        }
                    }

                    next = if ISA!(SA[a]) != ISAd!(SA[a]) {
                        tr_ilg(a - first + 1)
                    } else {
                        -1
                    };
                    a += 1;
                    if a < last {
                        crosscheck!("++a<last");
                        // CLEMENTINE
                        b = first;
                        v = (a - 1).0;
                        while b < a {
                            {
                                let SA_b = SA[b];
                                ISA!(SA_b) = v;
                            }
                            b += 1;
                        }
                    }

                    // push
                    if (budget.check((a - first).0)) {
                        crosscheck!("budget pass");
                        if (a - first) <= (last - a) {
                            crosscheck!("push {} {} {} {} {}", ISAd, a, last, -3, trlink);
                            stack.push(ISAd, a, last, -3, trlink);
                            ISAd += incr;
                            last = a;
                            limit = next;
                        } else {
                            if 1 < (last - a) {
                                crosscheck!(
                                    "push {} {} {} {} {}",
                                    ISAd + incr,
                                    first,
                                    a,
                                    next,
                                    trlink
                                );
                                stack.push(ISAd + incr, first, a, next, trlink);
                                first = a;
                                limit = -3;
                            } else {
                                ISAd += incr;
                                last = a;
                                limit = next;
                            }
                        }
                    } else {
                        crosscheck!("budget fail");
                        if 0 <= trlink {
                            crosscheck!("0<=trlink");
                            stack.items[trlink as usize].d = -1;
                        }
                        if 1 < (last - a) {
                            crosscheck!("1<(last-a)");
                            first = a;
                            limit = -3;
                        } else {
                            crosscheck!("1<(last-a) not");
                            if !stack
                                .pop(&mut ISAd, &mut first, &mut last, &mut limit, &mut trlink)
                                .is_ok()
                            {
                                return;
                            }
                            crosscheck!("1<(last-a) not post");
                            crosscheck!(
                                "were popped: ISAd={} first={} last={} limit={} trlink={}",
                                ISAd,
                                first,
                                last,
                                limit,
                                trlink
                            );
                        }
                    }
                } else {
                    crosscheck!("times pop");
                    if !stack
                        .pop(&mut ISAd, &mut first, &mut last, &mut limit, &mut trlink)
                        .is_ok()
                    {
                        return;
                    }
                    crosscheck!("times pop-post");
                    crosscheck!(
                        "were popped: ISAd={} first={} last={} limit={} trlink={}",
                        ISAd,
                        first,
                        last,
                        limit,
                        trlink
                    );
                } // end if first < last
            } // end if limit == -1, -2, or something else
            continue;
        } // end if limit < 0

        if (last - first) <= TR_INSERTIONSORT_THRESHOLD {
            crosscheck!("insertionsort last-first={}", last - first);
            tr_insertionsort(SA, ISAd, first, last);
            limit = -3;
            continue;
        }

        let old_limit = limit;
        limit -= 1;
        if (old_limit == 0) {
            crosscheck!(
                "heapsort ISAd={} first={} last={} last-first={}",
                ISAd,
                first,
                last,
                last - first
            );
            SA_dump!(&SA.range(first..last), "before tr_heapsort");
            tr_heapsort(ISAd, SA, first, (last - first).0);
            SA_dump!(&SA.range(first..last), "after tr_heapsort");

            // YOHAN
            a = last - 1;
            while first < a {
                // VINCENT
                x = ISAd!(SA[a]);
                b = a - 1;
                while (first <= b) && (ISAd!(SA[b])) == x {
                    SA[b] = !SA[b];

                    // iter (VINCENT)
                    b -= 1;
                }

                // iter (YOHAN)
                a = b;
            }
            limit = -3;
            crosscheck!("post-vincent continue");
            continue;
        }

        // choose pivot
        a = tr_pivot(SA, ISAd, first, last);
        crosscheck!("picked pivot {}", a);
        SA.swap(first, a);
        v = ISAd!(SA[first]);

        // partition
        tr_partition(SA, ISAd, first, first + 1, last, &mut a, &mut b, v);
        if (last - first) != (b - a) {
            crosscheck!("pre-nolwenn");
            next = if ISA!(SA[a]) != v { tr_ilg(b - a) } else { -1 };

            // update ranks
            // NOLWENN
            c = first;
            v = (a - 1).0;
            while c < a {
                {
                    let SAc = SA[c];
                    ISA!(SAc) = v;
                }
                c += 1;
            }
            if b < last {
                // ARTHUR
                c = a;
                v = (b - 1).0;
                while c < b {
                    {
                        let SAc = SA[c];
                        ISA!(SAc) = v;
                    }
                    c += 1;
                }
            }

            // push
            if (1 < (b - a)) && budget.check(b - a) {
                crosscheck!("a");
                if (a - first) <= (last - b) {
                    crosscheck!("aa");
                    if (last - b) <= (b - a) {
                        crosscheck!("aaa");
                        if 1 < (a - first) {
                            crosscheck!("aaaa");
                            crosscheck!("push {} {} {} {} {}", ISAd + incr, a, b, next, trlink);
                            stack.push(ISAd + incr, a, b, next, trlink);
                            crosscheck!("push {} {} {} {} {}", ISAd, b, last, limit, trlink);
                            stack.push(ISAd, b, last, limit, trlink);
                            last = a;
                        } else if 1 < (last - b) {
                            crosscheck!("aaab");
                            crosscheck!("push {} {} {} {} {}", ISAd + incr, a, b, next, trlink);
                            stack.push(ISAd + incr, a, b, next, trlink);
                            first = b;
                        } else {
                            crosscheck!("aaac");
                            ISAd += incr;
                            first = a;
                            last = b;
                            limit = next;
                        }
                    } else if (a - first) <= (b - a) {
                        crosscheck!("aab");
                        if 1 < (a - first) {
                            crosscheck!("aaba");
                            crosscheck!("push {} {} {} {} {}", ISAd, b, last, limit, trlink);
                            stack.push(ISAd, b, last, limit, trlink);
                            crosscheck!("push {} {} {} {} {}", ISAd + incr, a, b, next, trlink);
                            stack.push(ISAd + incr, a, b, next, trlink);
                            last = a;
                        } else {
                            crosscheck!("aabb");
                            crosscheck!("push {} {} {} {} {}", ISAd, b, last, limit, trlink);
                            stack.push(ISAd, b, last, limit, trlink);
                            ISAd += incr;
                            first = a;
                            last = b;
                            limit = next;
                        }
                    } else {
                        crosscheck!("aac");
                        crosscheck!("push {} {} {} {} {}", ISAd, b, last, limit, trlink);
                        stack.push(ISAd, b, last, limit, trlink);
                        crosscheck!("push {} {} {} {} {}", ISAd, first, a, limit, trlink);
                        stack.push(ISAd, first, a, limit, trlink);
                        ISAd += incr;
                        first = a;
                        last = b;
                        limit = next;
                    }
                } else {
                    crosscheck!("ab");
                    if (a - first) <= (b - a) {
                        crosscheck!("aba");
                        if 1 < (last - b) {
                            crosscheck!("abaa");
                            crosscheck!("push {} {} {} {} {}", ISAd + incr, a, b, next, trlink);
                            stack.push(ISAd + incr, a, b, next, trlink);
                            crosscheck!("push {} {} {} {} {}", ISAd, first, a, limit, trlink);
                            stack.push(ISAd, first, a, limit, trlink);
                            first = b;
                        } else if 1 < (a - first) {
                            crosscheck!("abab");
                            crosscheck!("push {} {} {} {} {}", ISAd + incr, a, b, next, trlink);
                            stack.push(ISAd + incr, a, b, next, trlink);
                            last = a;
                        } else {
                            crosscheck!("abac");
                            ISAd += incr;
                            first = a;
                            last = b;
                            limit = next;
                        }
                    } else if (last - b) <= (b - a) {
                        crosscheck!("abb");
                        if 1 < (last - b) {
                            crosscheck!("abba");
                            crosscheck!("push {} {} {} {} {}", ISAd, first, a, limit, trlink);
                            stack.push(ISAd, first, a, limit, trlink);
                            crosscheck!("push {} {} {} {} {}", ISAd + incr, a, b, next, trlink);
                            stack.push(ISAd + incr, a, b, next, trlink);
                            first = b;
                        } else {
                            crosscheck!("abbb");
                            crosscheck!("push {} {} {} {} {}", ISAd, first, a, limit, trlink);
                            stack.push(ISAd, first, a, limit, trlink);
                            ISAd += incr;
                            first = a;
                            last = b;
                            limit = next;
                        }
                    } else {
                        crosscheck!("abc");
                        crosscheck!("push {} {} {} {} {}", ISAd, first, a, limit, trlink);
                        stack.push(ISAd, first, a, limit, trlink);
                        crosscheck!("push {} {} {} {} {}", ISAd, b, last, limit, trlink);
                        stack.push(ISAd, b, last, limit, trlink);
                        ISAd += incr;
                        first = a;
                        last = b;
                        limit = next;
                    }
                }
            } else {
                crosscheck!("b");
                if (1 < (b - a)) && (0 <= trlink) {
                    crosscheck!("ba");
                    stack.items[trlink as usize].d = -1;
                }
                if (a - first) <= (last - b) {
                    crosscheck!("bb");
                    if 1 < (a - first) {
                        crosscheck!("bba");
                        crosscheck!("push {} {} {} {} {}", ISAd, b, last, limit, trlink);
                        stack.push(ISAd, b, last, limit, trlink);
                        last = a;
                    } else if 1 < (last - b) {
                        crosscheck!("bbb");
                        first = b;
                    } else {
                        crosscheck!("bbc");
                        if !stack
                            .pop(&mut ISAd, &mut first, &mut last, &mut limit, &mut trlink)
                            .is_ok()
                        {
                            return;
                        }
                    }
                } else {
                    crosscheck!("bc");
                    if 1 < (last - b) {
                        crosscheck!("bca");
                        crosscheck!("push {} {} {} {} {}", ISAd, first, a, limit, trlink);
                        stack.push(ISAd, first, a, limit, trlink);
                        first = b;
                    } else if 1 < (a - first) {
                        crosscheck!("bcb");
                        last = a;
                    } else {
                        crosscheck!("bcc");
                        if !stack
                            .pop(&mut ISAd, &mut first, &mut last, &mut limit, &mut trlink)
                            .is_ok()
                        {
                            return;
                        }
                        crosscheck!("bcc post");
                    }
                }
            }
        } else {
            crosscheck!("c");
            if budget.check(last - first) {
                crosscheck!("ca");
                limit = tr_ilg(last - first);
                ISAd += incr;
            } else {
                crosscheck!("cb");
                if 0 <= trlink {
                    crosscheck!("cba");
                    stack.items[trlink as usize].d = -1;
                }
                if !stack
                    .pop(&mut ISAd, &mut first, &mut last, &mut limit, &mut trlink)
                    .is_ok()
                {
                    return;
                }
                crosscheck!("cb post");
            }
        }
    } // end PASCAL
}

//------------------------------------------------------------------------------

//--------------------
// Function
//--------------------

/// Tandem repeat sort
pub fn trsort(ISA: SAPtr, SA: &mut SuffixArray, n: Idx, depth: Idx) {
    let mut ISAd: SAPtr;
    let mut first: SAPtr;
    let mut last: SAPtr;
    let mut t: Idx;
    let mut skip: Idx;
    let mut unsorted: Idx;
    let mut budget = Budget::new(tr_ilg(n) * 2 / 3, n);

    macro_rules! ISA {
        ($x: expr) => {
            SA[ISA + $x]
        };
    }

    // JERRY
    ISAd = ISA + depth;
    while (-n < SA[0]) {
        first = SAPtr(0);
        skip = 0;
        unsorted = 0;

        // PETER
        loop {
            t = SA[first];
            if (t < 0) {
                first -= t;
                skip += t;
            } else {
                if (skip != 0) {
                    SA[first + skip] = skip;
                    skip = 0;
                }
                last = SAPtr(ISA!(t) + 1);
                if (1 < (last - first)) {
                    budget.count = 0;
                    tr_introsort(ISA, ISAd, SA, first, last, &mut budget);
                    if (budget.count != 0) {
                        unsorted += budget.count;
                    } else {
                        skip = (first - last).0;
                    }
                } else if (last - first) == 1 {
                    skip = -1;
                }
                first = last;
            }

            // cond (PETER)
            if !(first < n) {
                break;
            }
        }

        if (skip != 0) {
            SA[first + skip] = skip;
        }
        if (unsorted == 0) {
            break;
        }

        // iter
        ISAd += ISAd - ISA;
    }
}
