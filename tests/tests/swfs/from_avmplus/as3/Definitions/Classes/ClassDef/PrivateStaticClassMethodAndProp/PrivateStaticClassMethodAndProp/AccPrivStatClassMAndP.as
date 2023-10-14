/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
 
 
 
 package PrivateStaticClassMethodAndProp {
 
      
   public class AccPrivStatClassMAndP {
      
    
    
   
       var array:Array;
       
       private var privDate:Date;
       
       public var pubBoolean:Boolean;
              
       static var statFunction:Function;
       
       private static var privStatString:String;
       
       public static var pubStatObject:Object;
       
       var finNumber:Number;
       
       public var pubFinArray:Array;
       
       static var finStatNumber:Number;
              
          
         
       // *****************
       // Default methods
       // *****************
   
       function getArray() : Array { return array; }
       function setArray( a:Array ) { array = a; }
       
               
        
        // *******************
    // private methods
    // *******************
       
       private function getPrivDate() : Date { return privDate; }
       private function setPrivDate( d:Date ) { privDate = d; }
       
       // wrapper function
       
       public function testGetSetPrivDate(d:Date) : Date {
            setPrivDate(d);
            return getPrivDate();
        }
       
       
   
       // *******************
       // public methods
       // *******************
   
       public function setPubBoolean( b:Boolean ) { pubBoolean = b; }
       public function getPubBoolean() : Boolean { return pubBoolean; }
       
          
   
       // *******************
       // static methods
       // *******************
   
       static function setStatFunction(f:Function) { statFunction = f; }
       static function getStatFunction() { return statFunction; }
        
       
   
       // **************************
       // private static methods
       // **************************
   
       private static function setPrivStatString(s:String) { privStatString = s; }
       private static function getPrivStatString() { return privStatString; }
       
       // wrapper function
       
       public function testGetSetPrivStatString(s:String) : String {
            setPrivStatString(s);
            return getPrivStatString();
        }
       
       
       
       // **************************
       // public static methods
       // **************************
   
       public static function setPubStatObject(o:Object) { pubStatObject = o; }
       public static function getPubStatObject() { return pubStatObject; }


   
       // *******************
       // final methods
       // *******************
   
       final function setFinNumber(n:Number) { finNumber = n; }
       final function getFinNumber() { return finNumber; }
           
       
       
       // *******************
       // public final methods
       // *******************
   
       public final function setPubFinArray(a:Array) { pubFinArray = a; }
       public final function getPubFinArray() { return pubFinArray; }
       
       
       
   
   
     
     
     // wrapper function
            
     public function testGetSetArray(a:Array) : Array {
           setArray(a);
           return getArray();
     }
        
     
     // wrapper function
            
     public function testGetSetPubBoolean(b:Boolean) : Boolean {
           setPubBoolean(b);
           return getPubBoolean();
     }
     
     // wrapper function
       
     public function testGetSetStatFunction(f:Function) : Function {
          setStatFunction(f);
           return getStatFunction();
     }
     
     
     
     // wrapper function
            
     public function testGetSetPubStatObject(o:Object) : Object {
          setPubStatObject(o);
           return getPubStatObject();
     }
     
     // wrapper function
       
     public function testGetSetFinNumber(n:Number) : Number {
           setFinNumber(n);
           return getFinNumber();
     }
     
     // wrapper function
       
     public function testGetSetPubArray(a:Array) : Array {
           setPubFinArray(a);
           return getPubFinArray();
     }
     



     
   
   } // AccPrivStatClassMAndP
   
}
