use std::borrow::Cow;

use flat_tree::{LineType, Tree, TreeLine};
use patterns::Pattern;
use screens::Screen;
use std::io::{self, Write};
use termion::{color, style};

pub trait TreeView {
    fn write_tree(&mut self, tree: &Tree) -> io::Result<()>;
    fn write_line_name(&mut self, line: &TreeLine, idx: usize, pattern: &Option<Pattern>) -> io::Result<()>;
}

impl TreeView for Screen {
    fn write_tree(&mut self, tree: &Tree) -> io::Result<()> {
        for y in 1..self.h - 1 {
            write!(
                self.stdout,
                "{}{}",
                termion::cursor::Goto(1, y),
                termion::clear::CurrentLine,
            )?;
            let line_index = (y - 1) as usize;
            if line_index >= tree.lines.len() {
                continue;
            }
            let line = &tree.lines[line_index];
            let selected = line_index == tree.selection;
            for depth in 0..line.depth {
                write!(
                    self.stdout,
                    "{}{}{}",
                    color::Fg(color::AnsiValue::grayscale(5)),
                    match line.left_branchs[depth as usize] {
                        true => match tree.has_branch(line_index + 1, depth as usize) {
                            true => match depth == line.depth - 1 {
                                true => "├─",
                                false => "│ ",
                            },
                            false => "└─",
                        },
                        false => "  ",
                    },
                    color::Fg(color::Reset),
                )?;
            }
            if selected {
                write!(
                    self.stdout,
                    "{}{}",
                    color::Bg(color::AnsiValue::grayscale(2)),
                    termion::clear::UntilNewline,
                );
            //} else {
            //    write!(self.stdout, " ");
            }
            self.write_line_name(line, line_index, &tree.pattern)?;
            write!(
                self.stdout,
                "{}{}{}",
                style::Reset,
                color::Fg(color::Reset),
                color::Bg(color::Reset),
            )?;
        }
        self.stdout.flush()?;
        Ok(())
    }

    fn write_line_name(
        &mut self,
        line: &TreeLine,
        idx: usize,
        pattern: &Option<Pattern>
    ) -> io::Result<()> {
        lazy_static! {
            static ref fg_reset: String = format!("{}", color::Fg(color::Reset)).to_string();
            static ref fg_dir: String = format!("{}", color::Fg(color::LightBlue)).to_string();
            static ref fg_link: String = format!("{}", color::Fg(color::LightMagenta)).to_string();
            static ref fg_match: String = format!("{}", color::Fg(color::Green)).to_string();
            static ref fg_reset_dir: String = format!("{}{}", &*fg_reset, &*fg_dir).to_string();
            static ref fg_reset_link: String = format!("{}{}", &*fg_reset, &*fg_link).to_string();
        }
        // TODO draw in red lines with has_error
        match &line.content {
            LineType::Dir => {
                if idx==0 {
                    write!(
                        self.stdout,
                        "{}{}{}",
                        style::Bold,
                        &*fg_dir,
                        &line.path.to_string_lossy(),
                    )?;
                } else {
                    write!(
                        self.stdout,
                        "{}{}{}",
                        style::Bold,
                        &*fg_dir,
                        decorated_name(&line.name, pattern, &*fg_match, &*fg_reset_dir),
                    )?;
                    if line.unlisted > 0 {
                        write!(self.stdout, " …",)?;
                    }
                }
            }
            LineType::File => {
                write!(
                    self.stdout,
                    "{}",
                    decorated_name(&line.name, pattern, &*fg_match, &*fg_reset),
                )?;
            }
            LineType::SymLink(target) => {
                write!(
                    self.stdout,
                    "{} {}->{} {}",
                    decorated_name(&line.name, pattern, &*fg_match, &*fg_reset),
                    &*fg_link,
                    &*fg_reset,
                    decorated_name(&target, pattern, &*fg_match, &*fg_reset),
                )?;
            }
            LineType::Pruning => {
                write!(
                    self.stdout,
                    "{} ... {} other files…",
                    style::Italic,
                    &line.unlisted,
                )?;
            }
        }
        Ok(())
    }
}

fn decorated_name<'a>(
    name: &'a str,
    pattern: &Option<Pattern>,
    prefix: &str,
    postfix: &str,
) -> Cow<'a, str> {
    if let Some(p) = pattern {
        if let Some(m) = p.test(name) {
            return Cow::Owned(m.wrap_matching_chars(name, prefix, postfix));
        }
    }
    Cow::Borrowed(name)
}
