<template>
<section class="t-footer bg_2 js_footer">
  <div class="c-m" style="width:1080px;margin:0 auto; position:releative;">

    <el-row style="position:relative;">
      <el-col :span="6">
        <h4 class="t-h">Resources</h4>
        <a class="t-a" target="_blank" href="https://teaproject.org">Website</a>
        <a class="t-a" target="_blank" href="https://github.com/tearust/teaproject/discussions">Discussions</a>
        <a class="t-a" target="_blank" href="https://github.com/tearust/teaproject/wiki">Wiki</a>
        <a class="t-a" target="_blank" href="https://github.com/tearust">Github</a>
      </el-col>

      <div style="position:absolute;right:0;bottom:0;">

        <b class="lg">
          <span>Block Height :</span> 
          <b>{{chain.current_block_hash ? chain.current_block : ''}}</b>
        </b>
      </div>
      
    </el-row>

    
    
    <el-divider></el-divider>
    <p style="margin:0 auto;text-align:center;font-size:16px;">Copyright © 2019-2021 <b>TeaProject.org</b> All Rights Reserved</p>
  </div>
</section>

</template>

<script>
import {mapGetters, mapState} from 'vuex';
import Base from '../workflow/Base';
import _ from 'lodash';
import utils from '../tea/utils';
export default {
  computed: {
    // ...mapGetters(['layer1_account']),
    ...mapState([
      'chain'
    ]),
  },
  data(){
    return {
      
    };
  },
  mounted(){
    
  }
}
</script>
<style lang="scss">
.t-footer{
  padding: 24px 0 40px;
  // position: fixed;
  // width: 100%;
  // bottom: 0;
  background: rgb(53, 166, 150);

  *{
    color: #fff;
  }

  .t-h{
    margin: 0 0 20px 0;
    text-align: left;
    font-size: 21px;
  }
  .t-a{
    font-size: 19px;
    display: table;
    &:hover{
      color: #ec7259;
    }
  }

  b.lg{
    display: flex;
    justify-content: space-between;
    width: 240px;
    font-size: 19px;
    font-weight: normal;

    span{
      width:150px;
      display: inline-block;
      text-align: right;
    }
    b{
      display: inline-flex;
      margin-left: 12px;
      flex: 1;
    }
  }
}
@media screen and (max-width : 600px) {
  .t-footer{
    padding-left: 20px;
    padding-right: 20px;
  }
}
</style>