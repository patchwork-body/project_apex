// Get the matched portion of a path based on a pattern or find common prefix between two paths
//
// This function handles multiple scenarios:
// 1. Pattern matching: "/{name}/{age}" with "/john/23/calculator" returns "/john/23"
// 2. Common prefix: "/users/profile/about" with "/users/profile/details" returns "/users/profile"
// 3. Exact match: "/users/profile" with "/users/profile" returns "/users/profile"
//
// The algorithm:
// - Compares segments from both inputs up to the shorter length
// - For each segment pair:
//   - If they match exactly, include in result
//   - If first segment is a parameter (starts with '{'), include the corresponding path segment
//   - Otherwise, stop and return what matched so far
// - If pattern is shorter than path, returns matched segments up to pattern length
// - If pattern is longer than path, returns "/" (no match)
pub(crate) fn get_matched_path(pattern: &str, path: &str) -> String {
    let pattern_segments: Vec<&str> = pattern.trim_start_matches('/').split('/').collect();
    let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    // If pattern has more segments than path, there's no match
    if pattern_segments.len() > path_segments.len() {
        return String::from("/");
    }

    let mut matched_segments = Vec::new();

    // Compare segments up to the pattern's length
    for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
        if pattern_seg == path_seg {
            // Exact match - include this segment
            matched_segments.push(*path_seg);
        } else if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
            // Parameter placeholder - include the corresponding path segment
            matched_segments.push(*path_seg);
        } else {
            // Segments don't match and pattern segment isn't a parameter - stop here
            break;
        }
    }

    if matched_segments.is_empty() {
        String::from("/")
    } else {
        format!("/{}", matched_segments.join("/"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_matched_path_unified() {
        // Test 1: Common prefix - two concrete paths differ in last segment
        assert_eq!(
            get_matched_path("/users/profile/about", "/users/profile/details"),
            "/users/profile"
        );

        // Test 2: Deeper paths with common prefix
        assert_eq!(
            get_matched_path("/api/v1/users/123/edit", "/api/v1/users/123/delete"),
            "/api/v1/users/123"
        );

        // Test 3: No common prefix (different from root)
        assert_eq!(get_matched_path("/home", "/about"), "/");

        // Test 4: Pattern with parameters matching exact path
        assert_eq!(
            get_matched_path("/{category}/items", "/products/items"),
            "/products/items"
        );

        // Test 5: Identical paths
        assert_eq!(
            get_matched_path("/users/profile", "/users/profile"),
            "/users/profile"
        );

        // Test 6: Pattern matching with longer path
        assert_eq!(
            get_matched_path("/{name}/{age}", "/john/23/calculator"),
            "/john/23"
        );

        // Test 7: Pattern longer than path (no match)
        assert_eq!(get_matched_path("/users/{id}/profile", "/users"), "/");

        // Test 8: Empty pattern and path
        assert_eq!(get_matched_path("", ""), "/");

        // Test 9: Root pattern with any path
        assert_eq!(get_matched_path("/", "/users/profile"), "/");

        // Test 10: Mixed parameters and literals
        assert_eq!(
            get_matched_path("/api/{version}/users/{id}", "/api/v2/users/123/edit"),
            "/api/v2/users/123"
        );

        // Test 11: Pattern with trailing slash
        assert_eq!(get_matched_path("/users/", "/users/profile"), "/users");

        // Test 12: Parameter at the end
        assert_eq!(get_matched_path("/users/{id}", "/users/123"), "/users/123");

        // Test 13: Multiple parameters in sequence
        assert_eq!(
            get_matched_path("/{org}/{repo}/{branch}", "/facebook/react/main/src"),
            "/facebook/react/main"
        );

        // Test 14: Partial match with literal difference
        assert_eq!(
            get_matched_path("/admin/users", "/admin/posts/create"),
            "/admin"
        );
    }
}
