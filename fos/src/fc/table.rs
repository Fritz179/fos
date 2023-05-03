use std::cell::{Cell, Ref, RefCell};

#[derive(Debug)]
pub struct Table<T> {
    items: RefCell<Vec<Option<T>>>,
    count: Cell<usize>,
}

impl<T> Table<T> {
    pub const fn new() -> Self {
        Table {
            items: RefCell::new(vec![]),
            count: Cell::new(0),
        }
    }

    pub fn get(&self, index: usize) -> Ref<T> {
        let items = self.items.borrow();

        let item = Ref::map(items, |x: &Vec<Option<T>>| &x[index]);

        Ref::map(item, |e| e.as_ref().expect("msg"))
    }

    pub fn len(&self) -> usize {
        self.count.get()
    }

    pub fn is_empty(&self) -> bool {
        self.count.get() == 0
    }

    pub fn next_free(&self) -> usize {
        let items = self.items.borrow();

        for i in 0..items.len() {
            if items[i].is_none() {
                return i;
            }
        }

        items.len()
    }

    pub fn add(&self, element: T) -> usize {
        let mut items = self.items.borrow_mut();

        // size growns by one
        self.count.set(self.count.get() + 1);

        for i in 0..items.len() {
            if items[i].is_none() {
                items[i] = Some(element);
                return i;
            }
        }

        items.push(Some(element));

        items.len() - 1
    }

    pub fn remove(&self, index: usize) -> Result<(), ()> {
        let mut items = self.items.borrow_mut();

        if index >= items.len() {
            return Err(());
        }

        if items.get(index).unwrap().is_some() {
            self.count.set(self.count.get() - 1);
            items[index] = None;
            return Ok(());
        }

        Err(())
    }

    pub fn exec(&self, index: usize, callback: Box<dyn Fn(&T)>) -> bool {
        let items = self.items.borrow();
        let item = items.get(index);

        if let Some(Some(item)) = item {
            callback(item);
            true
        } else {
            false
        }
    }

    pub fn filter(&self, callback: &dyn Fn(&T) -> bool) -> usize {
        let mut items = self.items.borrow_mut();

        for i in 0..items.len() {
            if let Some(item) = items.get(i).unwrap() {
                let keep = callback(item);

                if !keep {
                    self.count.set(self.count.get() - 1);
                    items[i] = None
                }
            }
        }

        self.count.get()
    }
}

#[cfg(test)]
mod tests {
    use super::Table;

    #[test]
    fn add_count() {
        let table = Table::new();

        assert_eq!(table.is_empty(), true);
        assert_eq!(table.len(), 0);

        let first = table.add(5);
        let second = table.add(6);

        assert_eq!(first, 0);
        assert_eq!(second, 1);
        assert_eq!(table.is_empty(), false);
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn remove_ok() {
        let table = Table::new();

        assert_eq!(table.is_empty(), true);
        assert_eq!(table.len(), 0);

        table.add(5);
        table.add(6);

        let ok_remove = table.remove(1 as usize);
        assert_eq!(ok_remove, Ok(()));
        assert_eq!(table.is_empty(), false);
        assert_eq!(table.len(), 1);

        let ok_remove = table.remove(0 as usize);
        assert_eq!(ok_remove, Ok(()));
        assert_eq!(table.is_empty(), true);
        assert_eq!(table.len(), 0);
    }

    #[test]
    fn remove_out_of_bounds() {
        let table = Table::new();

        table.add(5);
        table.add(6);

        let err_remove = table.remove(5 as usize);
        assert_eq!(err_remove, Err(()));
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn remove_twice() {
        let table = Table::new();

        table.add(5);
        table.add(6);

        let ok_remove = table.remove(1 as usize);
        let err_remove = table.remove(1 as usize);
        assert_eq!(ok_remove, Ok(()));
        assert_eq!(err_remove, Err(()));
        assert_eq!(table.len(), 1);
    }

    #[test]
    fn filter() {
        let table = Table::new();

        table.add(5);
        table.add(6);
        table.add(6);
        table.add(7);

        let size = table.filter(&|element: &i32| -> bool { *element != 6 });

        assert_eq!(size, 2);
        assert_eq!(table.len(), 2);
    }

    #[test]
    fn add_between() {
        let table = Table::new();

        table.add(5);
        table.add(6);
        table.add(7);

        table.remove(1 as usize).unwrap();

        let next = table.next_free();
        let second = table.add(8);

        assert_eq!(next, 1);
        assert_eq!(second, 1);
        assert_eq!(table.len(), 3);
    }
}
