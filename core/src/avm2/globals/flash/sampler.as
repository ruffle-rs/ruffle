package flash.sampler {
    public function clearSamples():void {
    }

    public function getGetterInvocationCount(obj:Object, name:QName):Number {
        return -1;
    }

    public function getInvocationCount(obj:Object, name:QName):Number {
        return -1;
    }

    public function getLexicalScopes(fun:Function):Array {
        return null;
    }

    public function getMasterString(str:String):String {
        return str;
    }

    public function getMemberNames(obj:Object, instanceNames:Boolean = false):Object {
        return [];
    }

    public function getSampleCount():Number {
        return 0;
    }

    public function getSamples():Object {
        return [];
    }

    public function getSavedThis(fun:Function):Object {
        return undefined;
    }

    public function getSetterInvocationCount(obj:Object, name:QName):Number {
        return -1;
    }

    public function getSize(param1:*):Number {
        return 0;
    }

    public function isGetterSetter(obj:Object, name:QName):Boolean {
        return false;
    }

    public function pauseSampling():void {
    }

    public function sampleInternalAllocs(everything:Boolean):void {
    }

    public function setSamplerCallback(fun:Function):void {
    }

    public function startSampling():void {
    }

    public function stopSampling():void {
    }
}
