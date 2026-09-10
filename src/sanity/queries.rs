// GROQ query strings for Sanity Content Lake.
//
// GROQ (Graph-Relational Object Queries) is Sanity's query language.
// Queries are stored as string constants for reuse and easy maintenance.

/// All posts, newest first, with tags dereferenced.
#[allow(dead_code)]
pub const ALL_POSTS: &str = r#"
    *[_type == "post"] | order(publishedAt desc) {
        title,
        slug,
        excerpt,
        publishedAt,
        mainImage,
        tags[]->{title, slug}
    }
"#;

/// Paginated post listing with `$offset` and `$end` parameters.
pub const POSTS_PAGINATED: &str = r#"
    *[_type == "post"] | order(publishedAt desc)[$offset...$end] {
        title,
        slug,
        excerpt,
        publishedAt,
        mainImage,
        tags[]->{title, slug}
    }
"#;

/// Single post by slug with full Portable Text body.
pub const POST_BY_SLUG: &str = r#"
    *[_type == "post" && slug.current == $slug][0] {
        title,
        slug,
        excerpt,
        body,
        publishedAt,
        mainImage,
        tags[]->{title, slug}
    }
"#;

/// Posts filtered by tag slug.
pub const POSTS_BY_TAG: &str = r#"
    *[_type == "post" && $tag in tags[]->slug.current] | order(publishedAt desc) {
        title,
        slug,
        excerpt,
        publishedAt,
        mainImage,
        tags[]->{title, slug}
    }
"#;

/// Text search across title and excerpt fields.
pub const SEARCH_POSTS: &str = r#"
    *[_type == "post" && (
        title match $query + "*"
        || excerpt match $query + "*"
    ] | order(publishedAt desc) {
        title,
        slug,
        excerpt,
        publishedAt,
        mainImage,
        tags[]->{title, slug}
    }
"#;

/// Total post count (returns a single number).
pub const POST_COUNT: &str = r#"count(*[_type == "post"])"#;

/// Post count for a specific tag.
pub const POST_COUNT_BY_TAG: &str =
    r#"count(*[_type == "post" && $tag in tags[]->slug.current])"#;

/// All tags ordered alphabetically.
#[allow(dead_code)]
pub const ALL_TAGS: &str = r#"
    *[_type == "tag"] | order(title asc) {
        title,
        slug,
        description
    }
"#;
