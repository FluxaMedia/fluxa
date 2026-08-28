import{n as y}from"./rolldown-runtime-CXHxssQy.js";import{Kt as L,Ut as I,Wt as m,yn as A}from"./engine-BA6xE4_h.js";import{D as g,U as E}from"./fetchPlanning-Bv00kLj3.js";import{r as v}from"./providerLibraries-Rkm0mn9D.js";import{g as C}from"./detailEffects--KZ5_Une.js";var P=y({fetchAniListCalendarItems:()=>N,pushLibraryStatusAniList:()=>$,pushWatchlistAniList:()=>S,syncAniListNow:()=>M}),f=`
  query ($userId: Int) {
    MediaListCollection(userId: $userId, type: ANIME) {
      lists {
        entries {
          status
          progress
          updatedAt
          media {
            id
            title { romaji english native }
            coverImage { large extraLarge }
            bannerImage
            episodes
            seasonYear
            genres
            nextAiringEpisode { airingAt episode }
          }
        }
      }
    }
  }
`,w="query { Viewer { id } }";async function M(t){const i=typeof t.token=="string"?t.token:void 0;if(!i)return{synced:!1,error:"AniList is not connected"};const a=t.profile,e=a?E(a):void 0,n=(await s(w,{},i))?.Viewer?.id;if(!n)return{synced:!1,error:"AniList account could not be loaded"};const d=((await s(f,{userId:n},i))?.MediaListCollection?.lists??[]).flatMap(u=>u.entries??[]).filter(u=>!!u?.media?.id),o=t.categories,c=t.dryRun===!0,p=t.readOnly===!0,r=await I(d,Date.now(),o,c);return r?(r.watching!=null&&!p&&await C({provider:"anilist",items:r.watching,profileKey:e}),!c&&!p&&await v("anilist",{watchlist:r.watchlist??[],watching:r.watching??[],completed:r.completed??[],dropped:r.dropped??[],favorites:[]},e),{synced:!0,provider:"anilist",continueWatchingCount:r.watchingCount,watchlistCount:r.watchlistCount,completedCount:r.completedCount,droppedCount:r.droppedCount,snapshot:{watchlist:r.watchlist??[],watching:r.watching??[],completed:r.completed??[],dropped:r.dropped??[],favorites:[]}}):{synced:!1,error:"AniList entries could not be processed"}}async function N(t){const i=(await s(w,{},t))?.Viewer?.id;if(!i)return[];const a=((await s(f,{userId:i},t))?.MediaListCollection?.lists??[]).flatMap(e=>e.entries??[]);return await A("providerCalendarItems",JSON.stringify({provider:"anilist",entries:a}))??[]}async function S(t,i,a){const e=h(t);if(e){if(i==="remove"){await b(e,a);return}await l(e,"PLANNING",a)}}async function $(t,i,a,e){const n=h(t);if(n){if(a==="remove"){await l(n,"CURRENT",e);return}i==="completed"?await l(n,"COMPLETED",e):i==="dropped"&&await l(n,"DROPPED",e)}}async function l(t,i,a){const e=await m(),n=await L(`anilist:${t}`,i)??{mediaId:t,status:i};await s(e?.saveMediaListEntry??"mutation ($mediaId: Int, $status: MediaListStatus) { SaveMediaListEntry(mediaId: $mediaId, status: $status) { id } }",n,a)}async function b(t,i){const a=await m(),e=(await s(a?.mediaListEntryLookup??"query ($mediaId: Int) { Media(id: $mediaId) { mediaListEntry { id } } }",{mediaId:t},i))?.Media?.mediaListEntry?.id;e&&await s(a?.deleteMediaListEntry??"mutation ($id: Int) { DeleteMediaListEntry(id: $id) { deleted } }",{id:e},i)}async function s(t,i,a,e=0){const n=await g("https://graphql.anilist.co",{method:"POST",headers:{Accept:"application/json","Content-Type":"application/json",Authorization:`Bearer ${a}`},body:JSON.stringify({query:t,variables:i})});if(n.status===429&&e===0){const o=Number(n.headers.get("Retry-After"));return await new Promise(c=>setTimeout(c,Math.min(6e4,Math.max(1e3,(Number.isFinite(o)?o:1)*1e3)))),s(t,i,a,1)}const d=await n.json();if(!n.ok||d.errors?.length)throw new Error(d.errors?.map(o=>o.message).filter(Boolean).join("; ")||`AniList request failed: HTTP ${n.status}`);return d.data??null}function h(t){const i=t.match(/^anilist:(\d+)/i);return i?Number(i[1]):null}export{M as a,S as i,N as n,$ as r,P as t};
