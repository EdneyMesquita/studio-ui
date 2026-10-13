//! Toolbars that fit their pane, like Rabbit Studio's container queries: as a view narrows (a split editor,
//! a small window) button labels collapse to icons with tooltips, then the less important items move into
//! the toolbar's "⋯" menu and fields shrink. Never overlap, never clip text mid-glyph.
//!
//! A toolbar measures what it would take at each level (button and segment widths come from cached galleys,
//! so this costs a few additions per frame) and draws the roomiest level that fits its width.

/// How much of a toolbar is shown, roomiest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fit {
    /// Everything, with labels.
    Full,
    /// Button labels collapse to icons (the label is the tooltip).
    Icons,
    /// Segments collapse too and the less important items go into "⋯".
    Compact,
}

impl Fit {
    /// The roomiest level whose width fits `avail`, given the widths a toolbar takes when full and when its
    /// buttons are icons.
    pub fn pick(avail: f32, full: f32, icons: f32) -> Self {
        if full <= avail {
            Self::Full
        } else if icons <= avail {
            Self::Icons
        } else {
            Self::Compact
        }
    }
    /// Button labels collapse to icons.
    pub fn icons(self) -> bool {
        self >= Self::Icons
    }
    /// Segments collapse and secondary items move into "⋯".
    pub fn compact(self) -> bool {
        self == Self::Compact
    }
}

/// The width of items laid out in a row `gap` apart.
pub fn row(widths: &[f32], gap: f32) -> f32 {
    let n = widths.iter().filter(|w| **w > 0.0).count();
    widths.iter().sum::<f32>() + gap * n.saturating_sub(1) as f32
}

/// Most items a [`flow`] places.
pub const FLOW_MAX: usize = 16;

/// Rows of a form-like toolbar that wraps (a flex row with `flex-wrap`): the row of each item when items of
/// these widths are placed left to right `gap` apart and wrap at `avail`, and the number of rows. An item
/// wider than a row gets a row of its own (and is drawn at most `avail` wide).
pub fn flow(widths: &[f32], gap: f32, avail: f32) -> ([u8; FLOW_MAX], usize) {
    let mut rows = [0u8; FLOW_MAX];
    let mut row = 0u8;
    let mut x = 0.0;
    let mut any = false;
    for (i, w) in widths.iter().take(FLOW_MAX).enumerate() {
        let need = if x > 0.0 { gap + w } else { *w };
        if x > 0.0 && x + need > avail {
            row += 1;
            x = *w;
        } else {
            x += need;
        }
        rows[i] = row;
        any = true;
    }
    (rows, if any { usize::from(row) + 1 } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_the_roomiest_level_that_fits() {
        assert_eq!(Fit::pick(900.0, 800.0, 500.0), Fit::Full);
        assert_eq!(Fit::pick(800.0, 800.0, 500.0), Fit::Full);
        assert_eq!(Fit::pick(600.0, 800.0, 500.0), Fit::Icons);
        assert_eq!(Fit::pick(400.0, 800.0, 500.0), Fit::Compact);
        assert!(Fit::Compact.icons() && Fit::Icons.icons() && !Fit::Full.icons());
        assert!(Fit::Compact.compact() && !Fit::Icons.compact());
    }

    #[test]
    fn row_counts_gaps_between_shown_items() {
        assert_eq!(row(&[100.0, 28.0, 0.0, 28.0], 4.0), 164.0);
        assert_eq!(row(&[], 4.0), 0.0);
        assert_eq!(row(&[50.0], 8.0), 50.0);
    }

    #[test]
    fn flow_wraps_at_the_width() {
        let (rows, n) = flow(&[100.0, 100.0, 100.0], 8.0, 1000.0);
        assert_eq!((&rows[..3], n), (&[0, 0, 0][..], 1));
        let (rows, n) = flow(&[100.0, 100.0, 100.0], 8.0, 210.0);
        assert_eq!((&rows[..3], n), (&[0, 0, 1][..], 2));
        // too wide for any row: alone on its own
        let (rows, n) = flow(&[50.0, 400.0, 50.0], 8.0, 200.0);
        assert_eq!((&rows[..3], n), (&[0, 1, 2][..], 3));
        assert_eq!(flow(&[], 8.0, 200.0).1, 0);
    }
}
