/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package PublicFunctionBody {

 class TestObjInner{
    public function noReturnNoParamsInner() { return "noReturnNoParams"; }
    public function noReturnParamsInner(s:String, b:Boolean) { return s; }
    public function noReturnCustomParamInner(c:Custom) { return new Custom(); }
    public function returnNoParamsInner():String { return "returnNoParams"; }
    public function returnParamsInner(s:String, b:Boolean):String { return s; }
    public function returnCustomNoParamsInner():Custom { return new Custom(); }
 }

}

