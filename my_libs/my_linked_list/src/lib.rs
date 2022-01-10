type Link<T> = Option<Box<Node<T>>>;

pub struct List<T> {
    head: Link<T>,
}

pub struct IntoIter<T>(List<T>);

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|box_node| &box_node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|box_node| &mut box_node.elem)
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }

    pub fn seek_f<F: Fn(&T) -> bool>(&self, comporator: F) -> Iter<'_, T> {
        let mut it = self.iter();
        while let Some(value) = it.peek() {
            if comporator(value) {
                break;
            }
            it.next();
        }
        return it;
    }

    pub fn seek_mut_f<F: Fn(&T) -> bool>(&mut self, comporator: F) -> IterMut<'_, T> {
        let mut it_mut = self.iter_mut();
        while let Some(value) = it_mut.peek_mut() {
            if comporator(value) {
                break;
            }
            it_mut.next();
        }
        return it_mut;
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();

        while let Some(mut boxed_node) = cur_link {
            cur_link = boxed_node.next.take();
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

impl<'a, T> Iter<'a, T> {
    pub fn peek(&self) -> Option<&T> {
        self.next.as_ref().map(|box_node| &box_node.elem)
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

impl<'a, T> IterMut<'a, T> {
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.next.as_mut().map(|box_node| &mut box_node.elem)
    }
}

impl<T> List<T>
where
    T: PartialEq,
{
    pub fn seek(&self, v: T) -> Iter<'_, T> {
        self.seek_f(|value| v.eq(value))
    }

    pub fn seek_mut(&mut self, v: T) -> IterMut<'_, T> {
        self.seek_mut_f(|value| v.eq(value))
    }
}

#[cfg(test)]
mod test {
    use crate::List;

    #[test]
    fn basics() {
        let mut list: List<i32> = List::new();

        // CHeck empty list behaviour
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check Normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more to check that pop didnt corrupt list
        list.push(4);
        list.push(5);

        // Check Normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);
        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        list.peek_mut().map(|value| *value = 42);
        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push("12345");
        list.push("Hello WORLD");
        list.push("lala");

        let mut iter = list.iter();

        assert_eq!(iter.next(), Some(&"lala"));
        assert_eq!(iter.next(), Some(&"Hello WORLD"));
        assert_eq!(iter.next(), Some(&"12345"));

        assert_eq!(list.peek(), Some(&"lala"));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push("12345");
        list.push("Hello WORLD");
        list.push("lala");

        let mut iter = list.iter_mut();

        *iter.next().unwrap() = "Hello WORLD";

        assert_eq!(list.peek(), Some(&"Hello WORLD"));
    }

    #[test]
    fn seek() {
        let mut list = List::new();
        list.push("12");
        list.push("123");
        list.push("1234");

        assert_eq!(list.seek("123").peek(), Some(&"123"));
        assert_eq!(list.seek("sdfsdf").peek(), None); // Test not found case
    }

    #[test]
    fn seek_mut() {
        let mut list = List::new();
        list.push("12");
        list.push("123");
        list.push("1234");

        *list.seek_mut("1234").peek_mut().unwrap() = "abc";

        assert!(list.seek_mut("abc").peek_mut().is_some());
    }
}
