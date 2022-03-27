<template>
<div class="tea-page">
  

  <PostMsg :channel="channel" />

  <!-- <el-button size="small" style="top:5px;" class="tea-refresh-btn" type="primary" plain icon="el-icon-refresh" circle @click="refreshList()"></el-button> -->
  
  
  <MessageList style="margin-top: 10px;" :list="list||[]" :channel="channel" />

  <!-- <el-divider /> -->
  
  
</div>
</template>
<script>
import { mapGetters, mapState } from 'vuex';
import Base from '../workflow/Base';
import {_, moment} from 'tearust_utils';
import utils from '../tea/utils';
import request from '../request';
// import TeaTable from '../components/TeaTable';
import {stringToHex, hexToString,} from 'tearust_layer1';
import bbs from './bbs';
import PostMsg from './PostMsg';
import MessageList from './MessageList';

export default {
  props: {
    channel: {
      type: String,
      required: true,
    }
  },
  components: {
    PostMsg, MessageList,
  },
  data(){
    return {
      list: null,

      loop: null,

      ch: null,
    };
  },
  computed: {
    ...mapGetters([
      'layer1_account'
    ]),
    ...mapState([
      'chain', 'user', 'bbs',
    ])
  },
  watch: {
    channel(val, old_val){
      this.ch = bbs.getChannel(this.channel);
      this.refreshList(true);
    }
  },
  async mounted(){
    this.wf = new Base();
    await this.wf.init();

    this.ch = bbs.getChannel(this.channel);
    
    await this.refreshList(true);
    this.startLoop();

    utils.mem.set('refresh-list__test', async ()=>{
      await this.refreshList(true);
      this.startLoop();
    });
    utils.mem.set('refresh-list__default', async ()=>{
      await this.refreshList(true);
      this.startLoop();
    });
    
  },

  methods: {

    async getDataList(){
      // if(!this.user){
      //   return [];
      // }

      const list = await bbs.loadMessageList(this.layer1_account.address, this.ch);
      // console.log(1111, new Date());

      return list;
    },

    async refreshList(show_loading=false){

      show_loading && this.$root.loading(true);
      // this.layoutListBefore();

      this.list = await this.getDataList();
      
      show_loading && this.$root.loading(false);
    },
    // layoutListBefore(){
    //   const el = $('.js_list');
    //   const hh = $(window).height()-250;
    //   // el.height(hh);
    //   el.css('max-height', hh);
    // },

    startLoop(){
      if(this.loop){
        clearTimeout(this.loop);
      }

      this.loop = setTimeout(async ()=>{
        await this.refreshList();

        // this.startLoop();
      }, 5000);


      // _.delay(async ()=>{
      //   if(!this.loop){
      //     return false;
      //   }

      //   const list = await this.getDataList(false);
      //   // const old_len = _.size(this.list);
      //   this.list = list;
      //   // if(old_len < _.size(list)){
      //   //   this.layoutListAfter();
      //   // }
      //   // this.startLoop();
      // }, 5000);
    },


    
  },
  
};

</script>
<style lang="scss">
.t-card{
  margin-top: 8px;
  cursor: pointer;

  &:hover{
    background: #f9f9f9;
  }
}
</style>