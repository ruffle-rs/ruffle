 
package
{
   import flash.display.MovieClip;
   import flash.utils.describeType;
   
   public class MainTimeline extends MovieClip
   {
       
      
      public function MainTimeline()
      {
         super();
         this.describeXMLNormalized(Described);
         this.describeXMLNormalized(ExtendedDescribed);
      }
      
      public function normalizeXML(data:XML, indent:uint = 0) : *
      {
         var i:* = undefined;
         var attr:* = undefined;
         var child:* = undefined;
         var childStr:* = undefined;
         var output:* = "";
         i = 0;
         while(i < indent)
         {
            output += " ";
            i++;
         }
         output += "<" + data.name();
         for each(attr in data.attributes())
         {
            output += " " + attr.name() + "=\"" + attr + "\"";
         }
         if(data.children().length() == 0)
         {
            return output + "/>";
         }
         output += ">\n";
         var childStrs:* = [];
         for each(child in data.children())
         {
            childStrs.push(this.normalizeXML(child,indent + 2));
         }
         childStrs.sort();
         for each(childStr in childStrs)
         {
            i = 0;
            while(i < indent)
            {
               output += " ";
               i++;
            }
            output += childStr;
            output += "\n";
         }
         i = 0;
         while(i < indent)
         {
            output += " ";
            i++;
         }
         return output + ("</" + data.name() + ">");
      }
      
      public function describeXMLNormalized(val:*) : *
      {
         trace(this.normalizeXML(describeType(val)));
      }
   }
}
 
package
{
   public class Described
   {
       
      
      [md]
      [mda("abcd")]
      [mdb(lastkey="lastvalue",thirdkey="thirdvalue",otherkey="othervalue",key="value")]
      public var something:Number;
      
      public var somethingElse:String;
      
      public function Described()
      {
         super();
      }
      
      [some_metadata]
      public function get onlyGetter() : Number
      {
         return 3;
      }
      
      [some_more_metadata]
      public function set virtual(value:Number) : *
      {
         trace("virtual setter called");
      }
      
      [more_metadata]
      [key_value_metadata(metadata="fun",more_key="some_value")]
      public function get virtual() : Number
      {
         return 4;
      }
      
      [meta_meta_data]
      public function set onlySetter(value:Number) : *
      {
         trace("only setter called");
      }
      
      [meta_meta_meta_data(recur="sion")]
      public function set toBeOverriddenOnlySetter(value:Number) : *
      {
         trace("only setter called");
      }
      
      [fn_metadata]
      [more("metadata")]
      public function noop() : void
      {
      }
   }
}
 
package
{
   public class ExtendedDescribed extends Described
   {
      
      [some_me]
      [ta_d("ata")]
      public static const const_prop = 4;
      
      [some_me2]
      [ta_d2("ata")]
      public static var var_prop = 5;
      
      [some_me_s2]
      [ta_d_s2("ataQ")]
      public var var_prop = 5;
       
      
      [some_me_s]
      [ta_d_s("ataB")]
      public const const_prop = 4;
      
      public function ExtendedDescribed()
      {
         super();
      }
      
      [metadata_here("hello!")]
      public function noop2() : Number
      {
         return 4;
      }
      
      [some]
      [more("meta")]
      [d(a="ta")]
      override public function get virtual() : Number
      {
         return 5;
      }
      
      [over]
      [ridde("n")]
      [meta(dat="a")]
      override public function set toBeOverriddenOnlySetter(val:Number) : *
      {
         trace("Overriden setter");
      }
      
      [last_metadata]
      override public function noop() : void
      {
      }
   }
}

