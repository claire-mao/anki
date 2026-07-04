// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

export function topicDetailsPath(topicId: string): string {
    return `/topics/${encodeURIComponent(topicId)}`;
}

export function practicePathForTopic(topicId: string): string {
    const normalized = topicId.trim();
    if (!normalized) {
        return "/practice";
    }
    return `/practice?topic=${encodeURIComponent(normalized)}`;
}

export function decodeTopicIdParam(topicId: string): string {
    return decodeURIComponent(topicId);
}
