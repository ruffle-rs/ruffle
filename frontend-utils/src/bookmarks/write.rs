use crate::bookmarks::{Bookmark, Bookmarks};
use crate::parse::DocumentHolder;
use crate::write::TableExt;
use toml_edit::{value, ArrayOfTables, Table};

pub struct BookmarksWriter<'a>(&'a mut DocumentHolder<Bookmarks>);

impl<'a> BookmarksWriter<'a> {
    pub fn new(bookmarks: &'a mut DocumentHolder<Bookmarks>) -> Self {
        Self(bookmarks)
    }

    fn with_underlying_table(&mut self, fun: impl FnOnce(&mut Bookmarks, &mut ArrayOfTables)) {
        self.0.edit(|values, toml_document| {
            let table = toml_document.get_or_create_array_of_tables("bookmark");
            fun(values, table)
        })
    }

    fn with_bookmark_table(&mut self, index: usize, fun: impl FnOnce(&mut Bookmarks, &mut Table)) {
        self.with_underlying_table(|values, array_of_tables| {
            let table = array_of_tables
                .get_mut(index)
                .expect("invalid bookmark index");
            fun(values, table)
        })
    }

    pub fn add(&mut self, bookmark: Bookmark) {
        self.with_underlying_table(|values, table| {
            let mut bookmark_table = Table::new();
            bookmark_table["url"] = value(bookmark.url.to_string());
            bookmark_table["name"] = value(&bookmark.name);
            table.push(bookmark_table);
            values.push(bookmark);
        })
    }

    pub fn set_url(&mut self, index: usize, url: url::Url) {
        self.with_bookmark_table(index, |values, table| {
            table["url"] = value(url.as_str());
            values[index].url = url;
        })
    }

    pub fn set_name(&mut self, index: usize, name: String) {
        self.with_bookmark_table(index, |values, table| {
            table["name"] = value(&name);
            values[index].name = name;
        })
    }

    pub fn remove(&mut self, index: usize) {
        self.with_underlying_table(|values, table| {
            table.remove(index);
            values.remove(index);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bookmarks::read_bookmarks;
    use url::Url;

    crate::define_serialization_test_helpers!(read_bookmarks, Bookmarks, BookmarksWriter);

    #[test]
    fn add_bookmark() {
        test(
            "",
            |writer| {
                writer.add(Bookmark {
                    url: Url::parse("file:///home/user/example.swf").unwrap(),
                    name: "example.swf".to_string(),
                })
            },
            "[[bookmark]]\nurl = \"file:///home/user/example.swf\"\nname = \"example.swf\"\n",
        );
        test("[[bookmark]]\nurl = \"file:///home/user/example.swf\"\n", |writer| writer.add(Bookmark {
            url: Url::parse("file:///home/user/another_file.swf").unwrap(),
            name: "another_file.swf".to_string(),
        }), "[[bookmark]]\nurl = \"file:///home/user/example.swf\"\n\n[[bookmark]]\nurl = \"file:///home/user/another_file.swf\"\nname = \"another_file.swf\"\n");
    }

    #[test]
    fn modify_bookmark() {
        test(
            "[[bookmark]]\nurl = \"file:///example.swf\"\n",
            |writer| writer.set_name(0, "Custom Name".to_string()),
            "[[bookmark]]\nurl = \"file:///example.swf\"\nname = \"Custom Name\"\n",
        );
        test(
            "[[bookmark]]\nurl = \"file:///example.swf\"\nname = \"example.swf\"",
            |writer| writer.set_url(0, Url::parse("https://ruffle.rs/logo-anim.swf").unwrap()),
            "[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\nname = \"example.swf\"\n",
        );
    }

    #[test]
    fn remove_bookmark() {
        test(
            "[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\n\n[[bookmark]]\nurl = \"file:///another_file.swf\"\n",
            |writer| {
                writer.remove(1);
            },
            "[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\nurl = \"file:///another_file.swf\"\n",
        );
        test("[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\n\n[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\n\n[[bookmark]]\nurl = \"invalid\"\n", |writer| {
            writer.remove(2);
        }, "[[bookmark]]\nurl = \"file://home/user/example.swf\"\n\n[[bookmark]]\n\n[[bookmark]]\nurl = \"invalid\"\n");

        // check if we can remove invalid entries.
        test("[[bookmark]]", |writer| writer.remove(0), "");
    }

    #[test]
    fn overwrite_invalid_bookmark_type() {
        test(
            "[bookmark]",
            |writer| {
                writer.add(Bookmark {
                    url: Url::parse("file:///test.swf").unwrap(),
                    name: "test.swf".to_string(),
                })
            },
            "[[bookmark]]\nurl = \"file:///test.swf\"\nname = \"test.swf\"\n",
        );

        test(
            "bookmark = 1010",
            |writer| {
                writer.add(Bookmark {
                    url: Url::parse("file:///test.swf").unwrap(),
                    name: "test.swf".to_string(),
                })
            },
            "[[bookmark]]\nurl = \"file:///test.swf\"\nname = \"test.swf\"\n",
        );
    }
}
