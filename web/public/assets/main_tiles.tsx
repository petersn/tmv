<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.8" tiledversion="1.8.0" name="main_tiles" tilewidth="32" tileheight="32" tilecount="64" columns="8">
 <image source="images/main_tiles.png" width="256" height="256"/>
 <tile id="0" type="nonsolid">
  <properties>
   <property name="name" value="spike"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" x="4" y="33">
    <polygon points="0,0 12,-26 25,1"/>
   </object>
  </objectgroup>
 </tile>
 <tile id="1" type="marker">
  <properties>
   <property name="name" value="coin"/>
  </properties>
 </tile>
 <tile id="2" type="marker">
  <properties>
   <property name="name" value="rare_coin"/>
  </properties>
 </tile>
 <tile id="3" type="nonsolid">
  <properties>
   <property name="name" value="shooter1"/>
  </properties>
 </tile>
 <tile id="4" type="nonsolid">
  <properties>
   <property name="name" value="save_left"/>
  </properties>
 </tile>
 <tile id="5" type="nonsolid"/>
 <tile id="8" type="marker">
  <properties>
   <property name="name" value="spawn"/>
  </properties>
 </tile>
 <tile id="9" type="nonsolid">
  <properties>
   <property name="name" value="water"/>
  </properties>
 </tile>
 <tile id="10" type="nonsolid">
  <properties>
   <property name="name" value="water"/>
  </properties>
 </tile>
</tileset>
