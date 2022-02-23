<template>
<div class="tea-page">
  <el-tabs tab-position="left" @tab-click="clickTab($event)" style="position:absolute;">
    <el-tab-pane key="receive" label="Received" :lazy="true">
    </el-tab-pane>
    <el-tab-pane key="send" label="Sent" :lazy="true">
    </el-tab-pane>
    
  </el-tabs>

  <div class="tea-page" style="margin-left: 130px;">
    <!-- <h4>{{type}}</h4> -->
    <el-button size="small" style="position:absolute;top:0;right:0;" @click="postNotification(null)"  type="primary">Compose new message</el-button>
    
    <div class="t-item" v-for="item of (list||[])" :key="item.id">
      <span v-if="type==='Received'" style="font-size: 13px; font-weight:bold; color:#8c8c8c;">
        From <b style="color:#35a696; margin-right: 10px;">{{item.sender}}</b>
        at <b style="color:#35a696;">{{item.utc}}</b> (Expired at <b style="color:#35a696;">{{item.utc_expired}}</b>)
      </span>
      <span v-if="type==='Send'" style="font-size: 13px; font-weight:bold; color:#8c8c8c;">
        To <b style="color:#35a696; margin-right: 10px;">{{item.to}}</b>
        at <b style="color:#35a696;">{{item.utc}}</b> (Expired at <b style="color:#35a696;">{{item.utc_expired}}</b>)
      </span>
      <p style="font-size: 18px; margin: 0; margin-top: 4px; padding-right: 70px; white-space: pre-line;">{{item.content}}</p>

      <div class="t-action" v-if="type==='Received'">
        <el-tooltip content="Reply message" placement="top" effect="light">
          <el-button size="mini" circle type="primary" plain icon="el-icon-position" @click="postNotification(item.sender)"></el-button>
        </el-tooltip>
        
      </div>

      
    </div>
    


  </div>
  
</div>
</template>

<script>
import { mapGetters, mapState } from 'vuex';
import {_} from 'tearust_utils';
import utils from '../tea/utils';
import Base from '../workflow/Base';
import bbs from './bbs';
export default {
  
  data(){
    return {
      type: 'Received',
      list: null,
    };
  },
  computed: {
    ...mapState([
      'user',
    ]),
    ...mapGetters([
      'layer1_account'
    ]),
  },
  watch: {
    async user(){
      this.refrersh_list();
    }
  },

  async mounted(){
    bbs.set_global_log(this);
    await this.refrersh_list();

  },
  methods: {
    async clickTab(e){
      this.type = e.label;
      
      await this.refrersh_list();
    },
    async refrersh_list(){
      this.$root.loading(true);
      let list = null;
      if(this.type === 'Received'){
        list = await bbs.getNotificationList(null, this.layer1_account.address);
      }
      else{
        list = await bbs.getNotificationList(this.layer1_account.address, null);
      }
      
      // console.log(11, list);
      this.list = list; //_.reverse(list);
        
      this.$root.loading(false);
    },
    async postNotification(to){
      await bbs.send_notification(this, to, async (f, rs)=>{
        if(f){
          this.$root.success("Send success");

          await this.refrersh_list();
        }
        else{
          this.$root.showError(rs);
        }
        
      });
    }
    
  }
  
}
</script>