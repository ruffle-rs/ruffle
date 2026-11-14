package {
    import flash.display.Sprite;
    import flash.net.ObjectEncoding;
    import flash.utils.ByteArray;
    import flash.utils.Dictionary;

    public class Test extends Sprite {
        public function Test() {
            var shared:Object = {value: "shared"};

            var repeatedArray:Array = [shared];

            var repeatedVector:Vector.<Object> = new Vector.<Object>();
            repeatedVector.push(shared);

            var repeatedDictionary:Dictionary = new Dictionary();
            repeatedDictionary["shared"] = shared;

            var original:Object = {
                array1: repeatedArray,
                array2: repeatedArray,
                vector1: repeatedVector,
                vector2: repeatedVector,
                dictionary1: repeatedDictionary,
                dictionary2: repeatedDictionary
            };

            var bytes:ByteArray = new ByteArray();
            bytes.objectEncoding = ObjectEncoding.AMF3;
            bytes.writeObject(original);
            bytes.position = 0;

            var result:Object = bytes.readObject();

            trace("Repeated Array reference: " + (result.array1 === result.array2));
            trace("Repeated Vector reference: " + (result.vector1 === result.vector2));
            trace("Repeated Dictionary reference: " + (result.dictionary1 === result.dictionary2));
            trace("Shared child reference: " +
                (result.array1[0] === result.vector1[0] &&
                 result.array1[0] === result.dictionary1["shared"]));
        }
    }
}
