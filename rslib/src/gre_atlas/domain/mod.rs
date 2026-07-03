// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

//! Canonical GRE domain model: sections, topic hierarchy, exam weights, and
//! coverage.

mod catalog;
mod coverage;
mod section;

pub use catalog::GreCatalog;
pub use catalog::TopicDef;
pub use catalog::TOPIC_TAG_PREFIX;
pub use coverage::compute_coverage;
pub use coverage::is_topic_covered;
pub use coverage::GreCoverage;
pub use coverage::GreSectionCoverage;
pub use section::GreSection;
