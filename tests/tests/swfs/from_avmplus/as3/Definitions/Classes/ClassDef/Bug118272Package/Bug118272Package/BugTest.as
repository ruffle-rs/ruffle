/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
// Bug 118272 - compile-time error to assign to a class identifier

// more elaborate variants:
package Bug118272Package {
    public class BugTest {
            public var thisError="no error";
            public var thisError1="no error";
            public var thisError2="no error";
            public var thisError3="no error";
            public var thisError4="no error";
            public function BugTest(){
              try{
         pubClass = 0;
                 }catch(e:ReferenceError){
                      thisError=e.toString();
                 }

              try{
         defClass = 0;
                 }catch(e1:ReferenceError){
                      thisError1=e1.toString();
                 }

              try{
         intClass = 0;
                 }catch(e2:ReferenceError){
                      thisError2=e2.toString();
                 }

              try{
         dynClass = 0;
                 }catch(e3:ReferenceError){
                      thisError3=e3.toString();
                 }

              try{
         finClass = 0;
                 }catch(e4:ReferenceError){
                      thisError4=e4.toString();
                 }
        }
    /*defClass = 1;
        intClass = 2;
        dynClass = 3;
        //expClass = 4;
        finClass = 5;*/
    }
}



