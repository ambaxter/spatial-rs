// Copyright 2016 spatial-rs Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

trait RetainPart<T, F>
    where F: FnMut(&T) -> bool
{
    fn retain_part(&mut self, mut f: F) -> usize;
}

trait RetainMutPart<T, F>
    where F: FnMut(&mut T) -> bool
{
    fn retain_mut_part(&mut self, mut f: F) -> usize;
}

pub trait RetainAndSplitOff<T, F>
    where F: FnMut(&T) -> bool
{
    fn retain_and_split_off(&mut self, mut f: F) -> Option<Vec<T>>;
}

pub trait RetainAndAppend<T, F>
    where F: FnMut(&T) -> bool
{
    fn retain_and_append(&mut self, m: &mut Vec<T>, mut f: F);
}

pub trait RetainMut<T, F>
    where F: FnMut(&mut T) -> bool
{
    fn retain_mut(&mut self, mut f: F);
}

impl<T, F> RetainPart<T, F> for Vec<T>
    where F: FnMut(&T) -> bool
{
    fn retain_part(&mut self, mut f: F) -> usize {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(&v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        del
    }
}

impl<T, F> RetainMutPart<T, F> for Vec<T>
    where F: FnMut(&mut T) -> bool
{
    fn retain_mut_part(&mut self, mut f: F) -> usize {
        let len = self.len();
        let mut del = 0;
        {
            let v = &mut **self;

            for i in 0..len {
                if !f(&mut v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        del
    }
}

#[allow(unused_mut)]
impl<T, F> RetainAndSplitOff<T, F> for Vec<T>
    where F: FnMut(&T) -> bool
{
    fn retain_and_split_off(&mut self, mut f: F) -> Option<Vec<T>> {
        let len = self.len();
        let del = self.retain_part(f);
        if del > 0 {
            return Some(self.split_off(len - del));
        }
        None
    }
}

#[allow(unused_mut)]
impl<T, F> RetainAndAppend<T, F> for Vec<T>
    where F: FnMut(&T) -> bool
{
    fn retain_and_append(&mut self, m: &mut Vec<T>, mut f: F) {
        let del = self.retain_part(f);
        if del > 0 {
            for i in 0..del {
                m.push(self.pop().unwrap());
            }
        }
    }
}

#[allow(unused_mut)]
impl<T, F> RetainMut<T, F> for Vec<T>
    where F: FnMut(&mut T) -> bool
{
    fn retain_mut(&mut self, mut f: F) {
        let len = self.len();
        let del = self.retain_mut_part(f);
        if del > 0 {
            self.truncate(len - del);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::PartialEq;

    trait ContainsAll<T>
        where T: PartialEq
    {
        fn contains_all(&self, that: &Vec<T>) -> bool;
    }

    impl<T> ContainsAll<T> for Vec<T> 
        where T: PartialEq
    {
        fn contains_all(&self, items: &Vec<T>) -> bool {
            for item in items {
                if !self.contains(item) {
                    return false;
                }
            }
            true
        }
    }

    #[test]
    fn retain_and_split_off() {
        let mut v = vec![1, 2, 3, 4, 5, 6];
        let left = vec![1, 2, 3];
        let right = vec![4, 5, 6];

        let split = v.retain_and_split_off(|x| *x < 4);
        assert!(left.contains_all(&v));
        assert!(!split.is_none());
        if let Some(split_v) = split {
            assert!(split_v.len() == 3);
            assert!(right.contains_all(&split_v));
        }
        
    }

    #[test]
    fn retain_and_append() {
        let mut v = vec![1, 2, 3, 4, 5, 6];
        let left = vec![1, 2, 3];
        let right = vec![4, 5, 6, 7];

        let mut appender = vec![7];

        v.retain_and_append(&mut appender, |x| *x < 4);
        assert!(v.len() == 3);
        assert!(left.contains_all(&v));
        assert!(appender.len() == 4);
        assert!(right.contains_all(&appender));
    }

    #[test]
    fn retain_mut() {
        let mut v = vec![1, 2, 3, 4, 5, 6];
        let left = vec![0, 1, 2];

        v.retain_mut(|x| {*x -=1; *x < 3});
        assert!(v.len() == 3);
        assert!(left.contains_all(&v));
    }
}