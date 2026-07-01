SELECT c.id,
  c.nid,
  c.did,
  c.ord,
  cast(c.mod AS integer),
  c.usn,
  c.type,
  c.queue,
  c.due,
  cast(c.ivl AS integer),
  c.factor,
  c.reps,
  c.lapses,
  c.left,
  c.odue,
  c.odid,
  c.flags,
  c.data,
  n.tags
FROM cards c
  INNER JOIN notes n ON c.nid = n.id
WHERE c.id IN (
    SELECT cid
    FROM search_cids
  )