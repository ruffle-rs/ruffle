var calledByChild = SharedObject.getLocal("so", null);

function childGetLocal(name, localPath) {
    return SharedObject.getLocal(name, localPath);
}

_level10.childLoaded(this);
