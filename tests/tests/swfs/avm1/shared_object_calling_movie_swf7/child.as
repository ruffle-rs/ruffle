var calledByChild = SharedObject.getLocal("so", null);

function childGetLocal(name, localPath) {
    return SharedObject.getLocal(name, localPath);
}

function callGetLocal(getter, name, localPath) {
    return getter(name, localPath);
}

_level0.childLoaded();
