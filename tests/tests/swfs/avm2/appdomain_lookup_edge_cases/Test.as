package {
    import flash.display.MovieClip;
    import flash.system.ApplicationDomain;
    import flash.utils.getDefinitionByName;

    public class Test extends MovieClip {
        public function Test() {
            trace(ApplicationDomain.currentDomain.hasDefinition("__AS3__.vec::Vector"));
            trace(ApplicationDomain.currentDomain.hasDefinition("Vector"));
            trace(ApplicationDomain.currentDomain.hasDefinition("Vector.<>"));
            trace(ApplicationDomain.currentDomain.hasDefinition("Vector.<void>"));
            trace(ApplicationDomain.currentDomain.hasDefinition("Vector.<null>"));
            trace(ApplicationDomain.currentDomain.hasDefinition("Vector.<Vector>"));
            trace(ApplicationDomain.currentDomain.hasDefinition("Vector.<__AS3__.vec::Vector>"));
            trace(ApplicationDomain.currentDomain.hasDefinition("Vector.<integerValue>"));

            try {
                trace(ApplicationDomain.currentDomain.getDefinition("Vector.<integerValue>"));
            } catch(e:VerifyError) {
                trace("error: " + e.errorID);
            }
            try {
                trace(getDefinitionByName("Vector.<integerValue>"));
            } catch(e:VerifyError) {
                trace("error: " + e.errorID);
            }

            trace(getDefinitionByName("JSON"));
            trace(ApplicationDomain.currentDomain.getDefinition("JSON"));
            trace(ApplicationDomain.currentDomain.hasDefinition("JSON"));

            try {
                trace(getDefinitionByName("::JSON"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }
            try {
                trace(ApplicationDomain.currentDomain.getDefinition("::JSON"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }
            try {
                trace(ApplicationDomain.currentDomain.hasDefinition("::JSON"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }

            try {
                trace(getDefinitionByName(".JSON"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }
            try {
                trace(ApplicationDomain.currentDomain.getDefinition(".JSON"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }
            try {
                trace(ApplicationDomain.currentDomain.hasDefinition(".JSON"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }

            trace(getDefinitionByName(".Math"));
            trace(ApplicationDomain.currentDomain.getDefinition(".Math"));
            trace(ApplicationDomain.currentDomain.hasDefinition(".Math"));

            trace(getDefinitionByName("::Math"));
            trace(ApplicationDomain.currentDomain.getDefinition("::Math"));
            trace(ApplicationDomain.currentDomain.hasDefinition("::Math"));

            try {
                trace(getDefinitionByName("flash.crypto::generateRandomBytes"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }
            try {
                trace(ApplicationDomain.currentDomain.getDefinition("flash.crypto::generateRandomBytes"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }
            try {
                trace(ApplicationDomain.currentDomain.hasDefinition("flash.crypto::generateRandomBytes"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }

            try {
                trace(getDefinitionByName("flash.crypto.generateRandomBytes"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }
            try {
                trace(ApplicationDomain.currentDomain.getDefinition("flash.crypto.generateRandomBytes"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }
            try {
                trace(ApplicationDomain.currentDomain.hasDefinition("flash.crypto.generateRandomBytes"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }

            try {
                trace(ApplicationDomain.currentDomain.getDefinition(".Vector"));
            } catch(e:Error) {
                trace("error: " + e.errorID);
            }
        }
    }

    public var integerValue:int = 15536007;
}
