package
{
    import flash.display.Sprite;

    public class Test extends Sprite
    {
    }
}

import flash.net.Socket;
import flash.errors.IOError;
import flash.utils.ByteArray;

var socket:Socket = new Socket();

try {
    trace("flush()");
    socket.flush();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("close()");
    socket.close();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readBoolean()");
    socket.readBoolean();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readByte()");
    socket.readByte();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readBytes()");
    var byteArray:ByteArray = new ByteArray();
    socket.readBytes(byteArray);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readDouble()");
    socket.readDouble();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readFloat()");
    socket.readFloat();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readInt()");
    socket.readInt();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readMultiByte()");
    socket.readMultiByte(10, "utf-8");
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readObject()");
    socket.readObject();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readShort()");
    socket.readShort();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readUnsignedByte()");
    socket.readUnsignedByte();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readUnsignedInt()");
    socket.readUnsignedInt();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readUnsignedShort()");
    socket.readUnsignedShort();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readUTF()");
    socket.readUTF();
} catch (e:IOError) {
    trace(e);
}

try {
    trace("readUTFBytes()");
    socket.readUTFBytes(10);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeBoolean()");
    socket.writeBoolean(false);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeByte()");
    socket.writeByte(127);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeBytes()");
    var byteArray2:ByteArray = new ByteArray();
    socket.writeBytes(byteArray2);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeDouble()");
    socket.writeDouble(10.0);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeFloat()");
    socket.writeFloat(56.0);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeInt()");
    socket.writeInt(2000000000);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeMultiByte()");
    socket.writeMultiByte("testing", "utf-8");
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeObject()");
    socket.writeObject(new Object());
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeShort()");
    socket.writeShort(58695);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeUnsignedInt()");
    socket.writeUnsignedInt(400000000);
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeUTF()");
    socket.writeUTF("testing");
} catch (e:IOError) {
    trace(e);
}

try {
    trace("writeUTFBytes()");
    socket.writeUTFBytes("testing");
} catch (e:IOError) {
    trace(e);
}