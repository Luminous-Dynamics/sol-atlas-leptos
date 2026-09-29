*** Begin Patch
*** Update File: sol-atlas-core/src/civilizational.rs
@@
 impl YearInterval {
@@
     pub fn contains(&self, year: i32) -> bool {
         self.from.is_none_or(|from| year >= from) && self.to.is_none_or(|to| year <= to)
     }
+
+    /// Returns whether two inclusive intervals share any year. Open bounds
+    /// represent unbounded time and therefore overlap any compatible range.
+    pub fn overlaps(&self, other: &Self) -> bool {
+        self.is_valid()
+            && other.is_valid()
+            && self.to.is_none_or(|to| other.from.is_none_or(|from| from <= to))
+            && other.to.is_none_or(|to| self.from.is_none_or(|from| from <= to))
+    }
 }
@@
-#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
 #[serde(rename_all = "snake_case")]
 pub enum ProjectionRef {
@@
 pub struct ProjectionAuditV1 {
@@
     pub source_snapshots: Vec<SourceSnapshotId>,
+    pub hypothesis_refs: Vec<HypothesisId>,
+    pub assessment: Option<AssessmentId>,
     pub temporal_scope: YearInterval,
@@
             source_snapshots: Vec::new(),
+            hypothesis_refs: Vec::new(),
+            assessment: snapshot.qualification.assessment.clone(),
             temporal_scope: snapshot.valid_time,
@@
             source_snapshots: transition.source_snapshots.clone(),
+            hypothesis_refs: transition.competing_hypotheses.clone(),
+            assessment: transition.assessment.clone(),
             temporal_scope: transition.event_time,
@@
             || self.source_snapshots.iter().any(|id| !id.is_valid())
+            || self.hypothesis_refs.iter().any(|id| !id.is_valid())
+            || self.assessment.as_ref().is_some_and(|id| !id.is_valid())
@@
 pub enum ProjectionError {
@@
     AuditWithoutEvidencePath,
+    InvalidSnapshot,
+    InvalidTransition,
 }
*** End Patch