use crate::store::{ItemList, ItemListTrait};
use askama::Template as AskamaTemplate;

#[derive(AskamaTemplate)]
#[template(path = "row.html")]
struct RowTemplate<'a> {
    id: &'a str,
    title: &'a str,
    completed: bool,
}

#[derive(AskamaTemplate)]
#[template(path = "itemsLeft.html")]
struct ItemsLeftTemplate {
    active_todos: usize,
}

pub struct Template {}

impl Template {
    pub fn item_list(items: ItemList) -> String {
        let mut output = String::from("");
        for item in items.iter() {
            let row = RowTemplate {
                id: &item.id,
                completed: item.completed,
                title: &item.title,
            };
            if let Ok(res) = row.render() {
                output.push_str(&res);
            }
        }
        output
    }

    pub fn item_counter(active_todos: usize) -> String {
        let items_left = ItemsLeftTemplate { active_todos };
        if let Ok(res) = items_left.render() {
            res
        } else {
            String::new()
        }
    }
}
