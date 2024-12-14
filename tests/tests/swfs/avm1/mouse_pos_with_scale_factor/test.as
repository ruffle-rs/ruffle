/*
  Test clip names:
    id: Identity transform
    zs: Zero-scaled
    ip: Integer position, 1:1 scale
    fp: Fractional position, 1:1 scale
    s42fp: Scaled to 42%, fractional position
    s133fp: Scaled to 133%, fractional position
    s133fpr15: Scaled to 133%, fractional position, rotated 15°
*/
  
zs._xscale = zs._yscale = 0

var oldX, oldY;

function dump() {

    if (_xmouse == oldX && _ymouse == oldY)
        return;

    oldX = _xmouse;
    oldY = _ymouse;

    trace('_root'+" "+_xmouse+" "+ _ymouse);
    var clips = [id, zs, ip, fp, s42fp, s133fp, s133fpr15];
    for (var i=0; i<clips.length; i++) {
        var clip = clips[i];
        trace(clip._name+" "+clip._xmouse+" "+clip._ymouse);
    }
    trace("");
}

function onEnterFrame() {
    dump();
}
