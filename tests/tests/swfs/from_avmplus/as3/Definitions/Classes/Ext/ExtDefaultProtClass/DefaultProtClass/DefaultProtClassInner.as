/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package DefaultProtClass {

  class DefaultProtClassInner {

    protected var protArray:Array;
    protected var protBoolean:Boolean;
    protected var protDate:Date;
    protected var protFunction:Function;
    protected var protMath:Math;
    protected var protNumber:Number;
    protected var protObject:Object;
    protected var protString:String;
    //protected var protSimple:Simple;

    protected static var protStatArray:Array;
    protected static var protStatBoolean:Boolean;
    protected static var protStatDate:Date;
    protected static var protStatFunction:Function;
    protected static var protStatMath:Math;
    protected static var protStatNumber:Number;
    protected static var protStatObject:Object;
    protected static var protStatString:String;
    //protected static var protStatSimple:Simple;

    // *****************************
    // to be overloaded
    // *****************************

    protected var protOverLoadVar;
    protected static var protStatOverLoadVar;

    // ****************
    // constructor
    // ****************

    function DefaultClassProt() {
    }

    // *******************
    // protected methods
    // *******************

    protected function setProtArray( a:Array ) { protArray = a; }
    protected function setProtBoolean( b:Boolean ) { protBoolean = b; }
    protected function setProtDate( d:Date ) { protDate = d; }
    protected function setProtFunction( f:Function ) { protFunction = f; }
    protected function setProtMath( m:Math ) { protMath = m; }
    protected function setProtNumber( n:Number ) { protNumber = n; }
    protected function setProtObject( o:Object ) { protObject = o; }
    protected function setProtString( s:String ) { protString = s; }
    //protected function setProtSimple( s:Simple ) { protSimple = s; }

    protected function getProtArray() : Array { return this.protArray; }
    protected function getProtBoolean() : Boolean { return this.protBoolean; }
    protected function getProtDate() : Date { return this.protDate; }
    protected function getProtFunction() : Function { return this.protFunction; }
    protected function getProtMath() : Math { return this.protMath; }
    protected function getProtNumber() : Number { return this.protNumber; }
    protected function getProtObject() : Object { return this.protObject; }
    protected function getProtString() : String { return this.protString; }
    //protected function getProtSimple() : Simple { return this.protSimple; }

    // **************************
    // protected static methods
    // **************************

    protected static function setProtStatArray(a:Array) { protStatArray=a; }
    protected static function setProtStatBoolean( b:Boolean ) { protStatBoolean = b; }

    protected static function getProtStatArray() { return protStatArray; }

    // ***************************
    // to be overloaded
    // ***************************

    protected function protOverLoad() { return "This is the parent class"; }
    protected static function protStatOverLoad() { return "This is the parent class"; }
  }

}
