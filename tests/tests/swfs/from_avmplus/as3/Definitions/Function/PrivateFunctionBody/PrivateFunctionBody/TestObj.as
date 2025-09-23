/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package PrivateFunctionBody {

    public class TestObj {
     private function noReturnNoParamsInner() { return "noReturnNoParams"; }
     private function noReturnParamsInner(s:String, b:Boolean) { return s; }
     private function noReturnCustomParamInner(c:Custom) { return new Custom(); }
     private function returnNoParamsInner():String { return "returnNoParams"; }
     private function returnParamsInner(s:String, b:Boolean):String { return s; }
     private function returnCustomNoParamsInner():Custom { return new Custom(); }
 
     public function noReturnNoParams() { return noReturnNoParamsInner(); }
     public function noReturnParams(s:String, b:Boolean) { return noReturnParamsInner(s,b); }
     public function noReturnCustomParam(c:Custom) { return noReturnCustomParamInner(c); }
     public function returnNoParams():String { return returnNoParamsInner(); }
     public function returnParams(s:String, b:Boolean):String { return returnParamsInner(s,b); }
     public function returnCustomNoParams():Custom { return returnCustomNoParamsInner(); }
    }


}

