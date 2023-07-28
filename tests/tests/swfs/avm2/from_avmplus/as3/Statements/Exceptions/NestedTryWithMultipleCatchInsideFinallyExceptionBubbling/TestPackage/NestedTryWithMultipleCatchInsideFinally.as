/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package TestPackage
{
    public class NestedTryWithMultipleCatchInsideFinally
    {
        public function NestedTryWithMultipleCatchInsideFinallyFunction():void
        {
            thisError = "no error";
            thisError1="no error";
            try{
                throw new TypeError();
   
                }catch(eo:ReferenceError){
                    thisError1 = "This is outer reference error:"+"  "+eo.toString();
                    //print(thisError1);
                }catch(eo2:ArgumentError){
                    thisError1="This is outer Argument Error"+eo2.toString();
                }catch(eo3:URIError){
                    thisError1="This is outer URI Error"+eo3.toString();
                }catch(eo4:EvalError){
                    thisError1="This is outer Eval Error"+eo4.toString();
                }catch(eo5:RangeError){
                    thisError1="This is outer Range Error"+eo5.toString();
                }catch(eo6:SecurityError){
                    thisError1="This is outer Security Error!!!"+eo6.toString();
                }finally{
                    try{
                       throw new TypeError();
                       }catch(ei:TypeError){
                           thisError="This is Inner Type Error:"+ei.toString();
                           //print(thisError);
                       }catch(ei1:ReferenceError){
                           thisError="Inner reference error:"+ei1.toString();
                       }catch(ei2:URIError){
                           thisError="This is inner URI Error:"+ei2.toString();
                       }catch(ei3:EvalError){
                           thisError="This is inner Eval Error:"+ei3.toString();
                       }catch(ei4:RangeError){
                           thisError="This is inner Range Error:"+ei4.toString();
                       }catch(ei5:SecurityError){
                           thisError="This is inner Security Error!!!"+ei5.toString();
                       }catch(ei6:ArgumentError){
                           thisError="This is inner Argument Error"+ei6.toString();
                       }finally{
               
                        }
                 }
           }
      }
}
