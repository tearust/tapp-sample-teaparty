<template>
<div>
  <div>
   
    <div style="position:relative;">
      <el-input type="textarea"
        :autosize="{ minRows: 1, maxRows: 4}"
        placeholder="What's happening?" 
        resize="none"
        @keydown.native="keyupHandler($event)"
        style="height: auto; width: 100%;" 
        v-model="form.msg">
      </el-input>

      <div style="position:relative; margin-top: 5px; margin-bottom: 20px; height:32px;">
        <el-select v-if="user && channel!=='test'" v-model="form.ttl" size="small" style="width:240px;">
          <el-option :key="1" label="14400 blocks (1 day) with 1 TEA" :value="14400" />
          <el-option :key="2" label="28800 blocks (2 day) with 2 TEA" :value="28800" />
          <el-option :key="3" label="43200 blocks (3 day) with 3 TEA" :value="43200" />
        </el-select>

        <span v-if="user && channel==='test'" style="font-size: 14px;color:#666;position:relative;top:-5px;">Post a 4800 blocks (8 hours) message for free.</span>

        <el-button v-if="user" style="position:absolute; width:150px; right:0;" size="small" type="primary" :disabled="!form.msg" @click="submitForm()">Post message</el-button>

        <el-button v-if="!user" style="position:absolute; width:150px; right:0;" size="small" type="primary" @click="showLoginModal()">Click to login</el-button>
      </div>
    </div>
    
    

    
  </div>

  <el-dialog
    :title="'Send message to '+channel"
    :visible="modal.visible"
    width="70%"
    :close-on-click-modal="false"
    custom-class="tea-modal"
    :destroy-on-close="true"
    @close="closeModal()"
  >

    <el-form :model="form" :rules="rules" ref="form" style="margin-top:0;">
      <el-form-item label="Message" prop="msg">
        <el-input placeholder="Input message here." type="textarea" v-model="form.msg"></el-input>
      </el-form-item>
    </el-form>
    

    <span slot="footer" class="dialog-footer">
      <el-button size="small" @click="closeModal()">Cancel</el-button>
      <el-button size="small" type="primary" @click="submitForm()">Send</el-button>
    </span>

  </el-dialog>
  
</div>
</template>
<script>
import { mapGetters, mapState } from 'vuex';
import Base from '../workflow/Base';
import {_} from 'tearust_utils';
import utils from '../tea/utils';
import request from '../request';
// import TeaTable from '../components/TeaTable';
import {stringToHex, hexToString,} from 'tearust_layer1';
import bbs from './bbs';
import user from './user';

export default {
  props: {
    channel: {
      type: String,
      required: true,
    }
  },
  data(){
    return {
      modal: {
        visible: false,
      },
      form: {
        msg: '',
        ttl: 14400,
      },
      rules: {
        msg: [{
          // required: true,
        }]
      }
    };
  },
  computed: {
    ...mapState(['user']),
    ...mapGetters([
      'layer1_account'
    ]),
  },
  async mounted(){
    bbs.set_global_log(this);
    // this.$root.loading(true);

    this.wf = new Base();
    await this.wf.init();

    // this.$root.loading(false);
  },
  methods: {
    async submitForm(){
      
      const cb = utils.mem.get('refresh-list__'+this.channel);
    
      const msg = this.form.msg;
      const ttl = this.channel==='test' ? 4800 : this.form.ttl;
      if(!msg){
        this.$root.showError('Please input message');
        return;
      }
      
      this.$root.loading(true);
      try{
        
        const rs = await bbs.sendMessage(this.layer1_account.address, msg, this.channel, ttl);
        
        this.$root.success();
        this.closeModal();
        if(cb){
          await cb();
        }
      }catch(e){
        console.log('post msg error =>', e);
        this.$root.showError(e);

        if(e === 'not_login'){
          await user.logout(this.layer1_account.address);
        }
        
        this.closeModal();
      }
      
      this.$root.loading(false);
    },

    closeModal(){
      this.modal.visible = false;
      this.form.msg = '';
    },
    openModal(){
      this.modal.visible = true;
    },
    showLoginModal(){
      
      user.showLoginModal(this);
    },
    async keyupHandler(e){
      if(e.keyCode !== 13) return;

      if(!e.shiftKey){
        if(this.user){
          await this.submitForm();
        }
        else{
          await this.showLoginModal();
        }
        e.preventDefault();  
        return false; 
      }


    }
    
  }
};

</script>
<style lang="scss">

</style>