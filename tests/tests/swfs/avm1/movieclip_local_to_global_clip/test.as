// Tests MovieClip.localToGlobal and globalToLocal with a movie clip that has
// `x` and `y` properties as the point, called directly, with
// Function.call and with Function.apply.

createEmptyMovieClip("container", getNextHighestDepth());
container._x = 100;
container._y = 50;

createEmptyMovieClip("point", getNextHighestDepth());

function reset() {
    point.x = 10;
    point.y = 20;
}

function show(label) {
    trace(label + ": " + point.x + ", " + point.y);
}

reset();
container.localToGlobal(point);
show("localToGlobal(clip)");
container.globalToLocal(point);
show("globalToLocal(clip)");

reset();
container.localToGlobal.call(container, point);
show("localToGlobal.call");
container.globalToLocal.call(container, point);
show("globalToLocal.call");

reset();
container.localToGlobal.apply(container, [point]);
show("localToGlobal.apply");
container.globalToLocal.apply(container, [point]);
show("globalToLocal.apply");
