<template>
  <div class="js_list t-list">

    <div class="t-item" v-for="item of (list||[])" :key="item.id">
      <span style="font-size: 13px; font-weight:bold; color:#8c8c8c;">
        {{item.sender}} - <b style="color:#35a696;">{{item.utc}}</b> (Expired at <b style="color:#35a696;">{{item.utc_expired}}</b>)
      </span>
      <p style="font-size: 18px; margin: 0; margin-top: 4px; padding-right: 70px; white-space: pre-line;">{{item.content}}</p>

      <div class="t-action">
        <el-tooltip v-if="channel!=='test' && user && user.address === item.sender" content="Extend message lifespan to 1200 more blocks." placement="top" effect="light">
          <el-button size="mini" circle type="primary" plain icon="el-icon-check" @click="clickExtendMessage(item)"></el-button>
        </el-tooltip>

        <el-tooltip v-if="
          (user && user.address === item.sender)
          || (user && bbs && bbs.tapp && user.address===bbs.tapp.owner)
        " content="Delete message" placement="top" effect="light">
          <el-button size="mini" circle type="danger" plain icon="el-icon-delete" @click="clickDeleteMessage(item)"></el-button>
        </el-tooltip>
        
      </div>
    </div>

    
  </div>
</template>
<script>
import $ from 'jquery';
import { mapGetters, mapState } from 'vuex';
import bbs from './bbs';
export default {
  props: {
    list: {
      type: Array,
      default: []
    },
    channel: {
      type: String,
      required: true,
    },
  },
  computed: {
    ...mapState([
      'user', 'bbs',
    ])
  },
  watch: {
    list(val){
      this.layoutListAfter();
    }
  },
  methods: {
    layoutListBefore(){
      const el = $('.js_list');
      const hh = $(window).height()-250;
      // el.height(hh);
      el.css('max-height', hh);
    },
    layoutListAfter(){
      this.$nextTick(()=>{
        const el = $('.js_list');
        el.scrollTop(99999);
      });
      
    },
    async clickExtendMessage(item){
      try{
        await this.$confirm('Are you sure to extend this message for 1200 blocks.', {
          title: 'Extend message',
          dangerouslyUseHTMLString: true,
        });
      }catch(e){
        return;
      }

      this.$root.loading(true);
      const cb = utils.mem.get('refresh-list__'+this.channel);
      await bbs.extend_message(this.user.address, item, this.channel);
      cb && cb();

      this.$root.loading(false);
    },
    async clickDeleteMessage(item){
      try{
        await this.$confirm('Are you sure to delete this message', {
          title: 'Info',
          dangerouslyUseHTMLString: true,
        });
      }catch(e){
        return;
      }

      this.$root.loading(true);
      const cb = utils.mem.get('refresh-list__'+this.channel);
      await bbs.delete_message(this.user.address, item, this.channel, this.bbs.tapp);
      cb && cb();

      this.$root.loading(false);
    }
  }
}
</script>
<style lang="scss">
.t-list{
  border: 1px solid #ececec;
  // border-top: none;
  // border-bottom: none;
  padding: 0 20px;
  min-height: 80px;
}
.t-item{
  position: relative;
  padding: 12px 0;
  border-bottom: 1px solid #ececec;

  &:last-child{
    border-bottom: none;
  }
}
.t-action{
  position: absolute;
  right: 0;
  top: 10px;
}
</style>