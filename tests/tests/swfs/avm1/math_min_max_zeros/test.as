function checkMinMaxOrdering(a, b) {
    trace("===== Checking " + a + ", " + b);
    trace(Math.max(a, b));
    trace(Math.min(a, b));
    trace(1.0/Math.max(a, b));
    trace(1.0/Math.min(a, b));
    trace(Number.POSITIVE_INFINITY == 1.0/Math.max(a, b));
    trace(Number.POSITIVE_INFINITY == 1.0/Math.min(a, b));
    trace(Number.NEGATIVE_INFINITY == 1.0/Math.max(a, b));
    trace(Number.NEGATIVE_INFINITY == 1.0/Math.min(a, b));
}

checkMinMaxOrdering(0.0, 1.0/Number.NEGATIVE_INFINITY);
checkMinMaxOrdering(0.0, 1.0/Number.POSITIVE_INFINITY);
checkMinMaxOrdering(1.0/Number.NEGATIVE_INFINITY, 0.0);
checkMinMaxOrdering(1.0/Number.POSITIVE_INFINITY, 0.0);
checkMinMaxOrdering(1.0/Number.POSITIVE_INFINITY, 1.0/Number.NEGATIVE_INFINITY);
checkMinMaxOrdering(1.0/Number.NEGATIVE_INFINITY, 1.0/Number.POSITIVE_INFINITY);

checkMinMaxOrdering(1.0/Number.NEGATIVE_INFINITY, 1.0/Number.NEGATIVE_INFINITY);
checkMinMaxOrdering(1.0/Number.POSITIVE_INFINITY, 1.0/Number.POSITIVE_INFINITY);

checkMinMaxOrdering(Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY);
checkMinMaxOrdering(Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY);
checkMinMaxOrdering(0.0, Number.NEGATIVE_INFINITY);
checkMinMaxOrdering(0.0, Number.POSITIVE_INFINITY);
checkMinMaxOrdering(Number.NEGATIVE_INFINITY, 0.0);
checkMinMaxOrdering(Number.POSITIVE_INFINITY, 0.0);
