function runGet(path) {
   trace("// get(\'" + path + "\')");
   trace(eval(path));
}

runGet("shape0");
runGet("clip1.shape1");
runGet("clip1.clip2.shape2");
runGet("clip1.clip2.clip3.shape3");
runGet("clip1.clip2.clip3.clip4.shape4");
runGet("_root/shape0");
runGet("clip1/shape1");
runGet("clip1/clip2.shape2");
runGet("clip1/clip2.clip3.shape3");
runGet("clip1/clip2.clip3.clip4.shape4");
runGet("_level0/shape0");
runGet("clip1/shape1");
runGet("clip1.clip2/shape2");
runGet("clip1.clip2.clip3/shape3");
runGet("clip1.clip2.clip3.clip4/shape4");
runGet("clip1.clip2/clip3.shape3");
runGet("clip1.clip2/clip3.clip4.shape4");
runGet("_level0:shape0");
runGet("clip1:shape1");
runGet("clip1.clip2:shape2");
runGet("clip1:clip2.clip3:shape3");
runGet("clip1.clip2:clip3.clip4:shape4");
runGet("shape0._parent");
runGet("clip1.shape1._parent");
runGet("clip1.clip2.shape2._parent");
runGet("clip1.clip2.shape2._parent.clip2");
runGet("clip1.clip2.shape2._parent/clip2");
