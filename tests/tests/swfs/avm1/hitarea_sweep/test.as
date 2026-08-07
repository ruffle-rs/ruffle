// Consolidated hitArea sweep. Follow the light-gray guides from the top row
// to the bottom row; empty-gap stops force roll-out wherever a same-owner
// zone must fire again. Every move has one Wait, and every down/up has a
// following Wait.
// Stops, in order (stage coordinates):
//
//    1. (0,0)
//    2. (70,70)       3. (190,70)      4. (250,70)
//    5. (310,70)      6. (430,70)      7. (550,70)
//    8. (670,70)
//    9. (70,190)     10. (150,190)    11. (230,190)
//   12. (350,190)    13. (470,190)    14. (590,190)
//   15. (710,190)
//   16. (70,310)     17. (190,310)    18. (310,310)
//   19. (490,310)    20. (570,310)    21. (650,310)
//   22. (570,310)    23. (490,310)
//   24. (70,430)     25. (300,430)    26. (440,430)
//   27. (300,430) click (xf arm)
//   28. (70,430) getter moves hit_xf  29. (570,430)
//   30. (440,430) getter moves btn_xfown
//   31. (690,430)
//   32. (190,550)    33. (310,550)    34. (650,550) down
//   35. (430,550)    36. (190,550)    37. (70,585) up (below the guide
//       line: the line is real vector art and would be the _droptarget)
//   38. (750,550) down                39. (610,550)
//   40. (490,550) up
//   41. (70,650)     42. (190,650)    43. (310,650)
//   44. (610,650)    45. (250,650) click (arm)
//   46. (610,650) re-entry (the getter removes btn_rm; the readback
//       traces a few frames later on its own)
//
// Zones:
//   row 1: chain_c, chain_b, btn_chain, hit_inv, hit_oinv, hit_zero
//   row 2: tf, gap, btn_tf, hit_mask, btn_mask, tfmask, btn_throw
//   row 3: hit_del, gap, btn_del, hit_ovr, gap, btn_ovr, gap, hit_ovr
//   row 4: hit_xf, btn_xf, arm gap, btn_xfown, gap,
//          hit_xf landing, btn_xfown landing
//   row 5: hitB, gap, dragA -> btnB -> hitB -> empty, dragB -> stopper
//   row 6: hit_rem, gap, btn_rem, hit_rm, arm gap, hit_rm
//
// lazy_getter stays a separate test because its prototype-level addProperty
// setter would consume every plain hitArea assignment in this SWF;
// remove_sibling stays separate as a precaution because Flash Player froze
// after a mid-pick sibling removal in earlier captures, and a second
// removal case in this capture would compound that risk.
stop();

function box(name, depth, color, x, y, w, h) {
    var mc = _root.createEmptyMovieClip(name, depth);
    mc.beginFill(color);
    mc.moveTo(x, y);
    mc.lineTo(x + w, y);
    mc.lineTo(x + w, y + h);
    mc.lineTo(x, y + h);
    mc.endFill();
    return mc;
}

function outline(name, depth, x, y, w, h) {
    var mc = _root.createEmptyMovieClip(name, depth);
    mc.lineStyle(1, 0x000000);
    mc.moveTo(x, y);
    mc.lineTo(x + w, y);
    mc.lineTo(x + w, y + h);
    mc.lineTo(x, y + h);
    mc.lineTo(x, y);
    return mc;
}

// Sweep top to bottom along these guides, left to right within each row.
var guide = _root.createEmptyMovieClip("guide", 1);
guide.lineStyle(1, 0xCCCCCC);
var rows = [70, 190, 310, 430, 550, 650];
for (var i = 0; i < rows.length; i++) {
    guide.moveTo(20, rows[i]);
    guide.lineTo(780, rows[i]);
}

var next_depth = 2;

// Row 1: chained, invisible, invisible-owner, and zero-scale cases.
box("chain_c", next_depth++, 0x00FF00, 30, 30, 80, 80);
box("chain_b", next_depth++, 0x0000FF, 150, 30, 80, 80);
box("btn_chain", next_depth++, 0xFF0000, 270, 30, 80, 80);
box("hit_inv", next_depth++, 0x00FF00, 390, 30, 80, 80);
hit_inv._visible = false;
outline("marker_inv", next_depth++, 390, 30, 80, 80);
box("hit_oinv", next_depth++, 0x00FF00, 510, 30, 80, 80);
box("hit_zero", next_depth++, 0x00FF00, 630, 30, 80, 80);
box("btn_inv", next_depth++, 0xFF0000, 730, 30, 60, 60);
box("btn_oinv", next_depth++, 0xFF0000, 730, 120, 60, 20);
box("btn_zero", next_depth++, 0xFF0000, 730, 240, 60, 20);

btn_chain.hitArea = chain_b;
chain_b.hitArea = chain_c;
btn_inv.hitArea = hit_inv;
btn_oinv._visible = false;
btn_oinv.hitArea = hit_oinv;
btn_zero._xscale = 0;
btn_zero.hitArea = hit_zero;

btn_chain.onRollOver = function() {
    trace("chain via " + (_root._xmouse < 130 ? "c" : (_root._xmouse < 250 ? "b" : "own")));
};
btn_inv.onRollOver = function() {
    trace("invisible");
};
btn_oinv.onRollOver = function() {
    trace("owner invisible");
};
btn_zero.onRollOver = function() {
    trace("zero scale");
};

// Row 2: TextField, mask, TextField-mask, and throwing-getter cases.
createTextField("tf", next_depth++, 30, 150, 80, 80);
tf.border = true;
tf.background = true;
tf.backgroundColor = 0x0000FF;
box("btn_tf", next_depth++, 0xFF0000, 190, 150, 80, 80);
box("hit_mask", next_depth++, 0x00FF00, 310, 150, 80, 80);
box("masked_mc", next_depth++, 0x888888, 370, 270, 60, 60);
masked_mc.setMask(hit_mask);
box("btn_mask", next_depth++, 0xFF0000, 430, 150, 80, 80);
createTextField("tfmask", next_depth++, 550, 150, 80, 80);
tfmask.border = true;
tfmask.background = true;
tfmask.backgroundColor = 0x00FFFF;
box("tfmask_masked", next_depth++, 0x888888, 370, 340, 60, 40);
tfmask_masked.setMask(tfmask);
box("btn_tfmask", next_depth++, 0xFF0000, 730, 150, 60, 40);
box("btn_throw", next_depth++, 0xFF0000, 670, 150, 80, 80);

btn_tf.hitArea = tf;
btn_mask.hitArea = hit_mask;
btn_tfmask.hitArea = tfmask;

btn_tf.onRollOver = function() {
    trace("textfield via " + (_root._xmouse < 170 ? "tf" : "own"));
};
btn_mask.onRollOver = function() {
    trace("mask via " + (_root._xmouse < 460 ? "hit" : "own"));
};
btn_tfmask.onRollOver = function() {
    trace("tfmask");
};
btn_throw.addProperty("hitArea", function() {
    throw "boom";
}, null);
btn_throw.onRollOver = function() {
    trace("throw via own");
};

// Row 3: delete the property, overwrite it, and revisit the old hit area.
box("hit_del", next_depth++, 0x00FF00, 30, 270, 80, 80);
box("btn_del", next_depth++, 0xFF0000, 270, 270, 80, 80);
btn_del.hitArea = hit_del;
var del_rollovers = 0;
btn_del.onRollOver = function() {
    del_rollovers++;
    trace("delete via " + (_root._xmouse < 130 ? "hit" : "own"));
    if (del_rollovers == 1) {
        delete btn_del.hitArea;
    }
};

box("hit_ovr", next_depth++, 0x00FF00, 450, 270, 80, 80);
box("btn_ovr", next_depth++, 0xFF0000, 610, 270, 80, 80);
btn_ovr.hitArea = hit_ovr;
var ovr_rollovers = 0;
btn_ovr.onRollOver = function() {
    ovr_rollovers++;
    trace("overwrite via " + (_root._xmouse < 550 ? "hit" : "own"));
    if (ovr_rollovers == 1) {
        btn_ovr.hitArea = 5;
    }
};

// Row 4: a transform mutated inside the getter takes effect on the very
// pick that ran the getter (unlike removal, which takes effect on the
// next pick). btn_xf's getter moves hit_xf away when the armed mouse
// enters it: no rollover fires on that pick, only at the new position.
// btn_xfown's getter moves btn_xfown itself and returns undefined, so
// its own shape is tested, likewise at the new position immediately.
// Both moves are horizontal, within this row.
box("hit_xf", next_depth++, 0x00FF00, 30, 390, 80, 80);
box("btn_xf", next_depth++, 0xFF0000, 150, 390, 80, 80);
box("btn_xfown", next_depth++, 0x0000FF, 400, 390, 80, 80);

var xf_armed = false;
var xf_moved = false;
var xfown_moved = false;

btn_xf.addProperty("hitArea", function() {
    if (xf_armed && !xf_moved && _root._xmouse < 130 && _root._ymouse > 380 && _root._ymouse < 480) {
        xf_moved = true;
        trace("moving hit_xf");
        _root.hit_xf._x += 500;
    }
    return _root.hit_xf;
}, null);
btn_xf.onRollOver = function() {
    trace("xf via " + (_root._xmouse < 300 ? "old" : "new"));
};

btn_xfown.addProperty("hitArea", function() {
    if (xf_armed && !xfown_moved && _root._xmouse > 380 && _root._xmouse < 500 && _root._ymouse > 380 && _root._ymouse < 480) {
        xfown_moved = true;
        trace("moving btn_xfown");
        _root.btn_xfown._x += 250;
    }
    return undefined;
}, null);
btn_xfown.onRollOver = function() {
    trace("xfown at " + (_root._xmouse < 550 ? "old" : "new"));
};

// Row 5: _droptarget ignores hitArea, then stopDrag runs inside a getter.
box("hitB", next_depth++, 0x00FF00, 150, 510, 80, 80);
box("btnB", next_depth++, 0xFF0000, 390, 510, 80, 80);
box("dragA", next_depth++, 0x0000FF, 630, 530, 40, 40);
box("dragB", next_depth++, 0x0000FF, 730, 530, 40, 40);
box("stopper", next_depth++, 0xFF0000, 570, 510, 40, 40);

btnB.hitArea = hitB;
var dt_announced = false;
btnB.onRollOver = function() {
    if (!dt_announced) {
        dt_announced = true;
        trace("rollover via hitB");
    }
};

var dragging_a = false;
var last_dt = "";
dragA.onPress = function() {
    this.startDrag(false);
    trace("drag start");
    dragging_a = true;
};
dragA.onRelease = function() {
    this.stopDrag();
    trace("drag end");
    dragging_a = false;
};
dragA.onReleaseOutside = dragA.onRelease;

var dragging_b = false;
dragB.onPress = function() {
    this.startDrag(false);
    trace("drag2 start");
    dragging_b = true;
};
dragB.onRelease = function() {
    trace("drag2 alive: " + dragging_b);
};
dragB.onReleaseOutside = dragB.onRelease;

stopper.addProperty("hitArea", function() {
    if (dragging_b && _root._xmouse < 650) {
        dragging_b = false;
        trace("drag2 stopped");
        stopDrag();
    }
    return undefined;
}, null);
stopper.onRollOver = function() {};

// Row 6: removed hit area and the owner-removal finale. This row is
// choreographed last: in captures, Flash Player sometimes stops
// delivering mouse input at some point after a mid-pick removal (the
// freeze remove_sibling's header documents), so nothing else may
// depend on events after this row's removal.
box("hit_rem", next_depth++, 0x00FF00, 30, 610, 80, 80);
outline("marker_rem", next_depth++, 30, 610, 80, 80);
box("btn_rem", next_depth++, 0xFF0000, 270, 610, 80, 80);
btn_rem.hitArea = hit_rem;
var rem_rollovers = 0;
btn_rem.onRollOver = function() {
    rem_rollovers++;
    trace("removed via " + (_root._xmouse < 130 ? "hit" : "own"));
    if (rem_rollovers == 1) {
        hit_rem.removeMovieClip();
    }
};

box("hit_rm", next_depth++, 0x00FF00, 570, 610, 80, 80);
box("btn_rm", next_depth++, 0xFF0000, 690, 610, 80, 80);
var rm_armed = false;
var rm_done = false;
// The removed-owner readback runs on a short frame delay instead of a
// click, so it cannot be lost to the input freeze described above.
var rm_readback = 6;
btn_rm.addProperty("hitArea", function() {
    if (rm_armed && !rm_done && _root._xmouse > 560 && _root._xmouse < 660 && _root._ymouse > 600) {
        rm_done = true;
        trace("removing owner");
        this.removeMovieClip();
    }
    return _root.hit_rm;
}, null);
btn_rm.onRollOver = function() {
    trace("rm via hit");
};


// Click-only assertions; drag clicks fall between the dispatcher's y bands.
onMouseDown = function() {
    var mx = _root._xmouse;
    var my = _root._ymouse;
    if (my > 380 && my < 480) {
        if (!xf_armed) {
            xf_armed = true;
            trace("xf armed");
        }
    } else if (my > 600 && mx > 140 && mx < 360) {
        rm_armed = true;
        trace("armed");
    }
};

trace("ready");

var frames = 0;
onEnterFrame = function() {
    frames++;
    if (rm_done && rm_readback-- == 0) {
        trace("btn_rm after: " + btn_rm);
    }
    if (dragging_a) {
        var dt = dragA._droptarget;
        if (dt != last_dt) {
            last_dt = dt;
            trace("dt: " + (dt == "" ? "(none)" : dt));
        }
    }
    if (frames >= 479) {
        fscommand("quit");
    }
};
