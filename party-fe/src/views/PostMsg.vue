<template>
<div>
  <div>
   
    <div style="position:relative;">
      <el-input type="textarea"
        :autosize="{ minRows: 1, maxRows: 4}"
        placeholder="What's happening?" 
        resize="none"
        @keydown.native="keyupHandler($event)"
        style="height: auto;" 
        v-model="form.msg">
      </el-input>

      <el-button v-if="user" style="position:absolute; width:120px; height: 32px; bottom:1px; left: 810px;" size="mini" type="primary" :disabled="!form.msg" @click="submitForm()">Post message</el-button>

      <el-button v-if="!user" style="position:absolute; width:120px; height: 32px; bottom:1px; left: 810px;" size="mini" type="primary" @click="showLoginModal()">Click to login</el-button>
    </div>
    <div style="text-align:right;font-size:12px; color: #9c9c9c;">Post message will be paid in epoch 7.</div>
    

    
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
    // this.$root.loading(true);

    this.wf = new Base();
    await this.wf.init();

    // this.$root.loading(false);
  },
  methods: {
    async submitForm(){
      const cb = utils.mem.get('refresh-list__'+this.channel);
    
      const msg = this.form.msg;
      if(!msg){
        this.$root.showError('Please input message');
        return;
      }
      
      this.$root.loading(true);
      try{
        bbs.setLog((msg)=>{
          bbs.top_log(msg);
        });
        const rs = await bbs.sendMessage(this.layer1_account.address, msg, this.channel);
        
        this.$message.success('success.');
        this.closeModal();
        if(cb){
          await cb();
        }
      }catch(e){
        this.$root.showError(e);

        await user.logout(this.layer1_account.address);
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