/// Review vs Skip labels the primary action on every prepare boundary.
export function actionVerb(skipReview: boolean): "Apply" | "Review" {
  return skipReview ? "Apply" : "Review";
}

export function submitHint(skipReview: boolean, reason: string): string {
  if (reason) return reason;
  return skipReview
    ? "Skip is on — this writes as soon as you apply."
    : "Nothing is written until you apply the review.";
}
