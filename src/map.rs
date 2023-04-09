use std::cell::RefCell;

pub struct Table<T> {
    items: RefCell<Vec<Option<T>>>
}

impl<'a, T> Table<T> {
    fn new() -> Self {
        Table { 
            items: RefCell::new(vec![])
         }
    }

    fn add(&self, element: T) -> u32 {
        let mut items = self.items.borrow_mut();

        for i in 0..items.len() {
            if let None = items[i] {
                items[i] = Some(element);
                return i as u32;
            }
        }

        items.push(Some(element));
        let index = items.len() - 1;

        return index as u32;
    }

    fn exec(self, index: usize, callback: Box<dyn Fn(&T)>) -> bool {
        let items = self.items.borrow();
        let item = items.get(index);

        // let items: &'a RefCell<Vec<Option<T>>>  = &self.items;
        // let items: Ref<Vec<Option<T>>> = items.borrow();
        // let value: Option<&'a Option<T>> = items.get(index);

        if let Some(item) = item {
            if let Some(item) = item {
                callback(item);
                return true 
            } 
        } 
    
        return false
    }
}