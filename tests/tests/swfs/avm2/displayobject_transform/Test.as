package
{
   import flash.display.*;
   import flash.geom.Transform;
   import flash.text.*;

   public dynamic class Test extends MovieClip
   {

      public function Test()
      {
         super();
         addFrameScript(0,this.frame1);
      }

      public function run(stage:Stage)
      {
         var firstTransform;
         var secondTransform;
         var tempMat;
         var tempColor;
         var otherMat;
         var otherColor;
         var printTransform = function(trans:Transform)
         {
            trace("colorTransform=" + trans.colorTransform);
            trace("matrix=" + trans.matrix);
            trace("concatenatedMatrix=" + trans.concatenatedMatrix);
         };
         var checkQuals = function(stage:Stage, child:DisplayObject)
         {
            var i = undefined;
            var qual = undefined;
            var quals = ["best","high","16x16","16x16linear","8x8","8x8linear","low","medium"];
            for(i in quals)
            {
               qual = quals[i];
               stage.quality = qual;
               trace(qual + " TextField " + child.transform.concatenatedMatrix);
               trace(qual + " stage" + stage.transform.concatenatedMatrix);
            }
         };
         var firstParent:Sprite = new Sprite();
         var secondParent:Sprite = new Sprite();
         var child:TextField = new TextField();
         firstParent.addChild(child);
         trace("Checking stage qualities with non-stage child");
         checkQuals(stage,child);
         stage.addChild(firstParent);
         trace("");
         trace("Checking stage qualities with child on stage");
         checkQuals(stage,child);
         trace("// child.transform == child.transform");
         trace(child.transform == child.transform);
         firstTransform = child.transform;
         secondTransform = child.transform;
         trace("// firstTransform");
         printTransform(firstTransform);
         trace("// secondTransform");
         printTransform(secondTransform);
         firstTransform.matrix.a = 99;
         firstTransform.colorTransform.redOffset = 249;
         trace("// firstTransform after no-op modifications");
         printTransform(firstTransform);
         tempMat = firstTransform.matrix;
         tempMat.a = 42;
         firstTransform.matrix = tempMat;
         trace("// firstTransform after matrix modification");
         printTransform(firstTransform);
         trace("// secondTransform");
         printTransform(secondTransform);
         tempColor = child.transform.colorTransform;
         tempColor.redOffset = 12;
         child.transform.colorTransform = tempColor;
         trace("// firstTransform after color modification");
         printTransform(firstTransform);
         trace("// secondTransform");
         printTransform(secondTransform);
         otherMat = secondParent.transform.matrix;
         otherColor = secondParent.transform.colorTransform;
         otherMat.a = otherMat.b = otherMat.c = otherMat.d = 42;
         otherColor.redMultiplier = otherColor.greenMultiplier = otherColor.blueMultiplier = 3;
         secondParent.transform.matrix = otherMat;
         secondParent.transform.colorTransform = otherColor;
         secondParent.addChild(child);
         trace("// firstTransform after setting parent");
         printTransform(firstTransform);
         trace("// secondTransform after setting parent");
         printTransform(secondTransform);
         firstParent.addChild(secondParent);
         stage.addChild(firstParent);
         trace("// firstTransform after indirectly added to stage");
         printTransform(firstTransform);
         trace("// secondTransform after indirectly added to stage");
         printTransform(secondTransform);
         stage.removeChild(firstParent);
         trace("// firstTransform after indirectly removed from stage");
         printTransform(firstTransform);
         trace("// secondTransform after indirectly removed forrm stage");
         printTransform(secondTransform);
      }

      internal function frame1() : *
      {
         this.run(stage);
      }
   }
   
}