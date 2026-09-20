# Jev — RAG reranking (third-party walkthrough)

**Standing:** external third-party material — **recorded, not adopted.** Every
number, latency, cost and accuracy figure below is the presenter's claim from
the video, unverified in this repository. Reuse in any argument here requires
measurement on our own corpora under the standing claim policy
(`../specimens.json` → `claim_policy`).

| | |
|---|---|
| Title | JEV: RAG Reranking for 1/10th the Cost |
| Channel | Prompt Engineering |
| Video id | `UhGH8cNG0qs` |
| URL | https://youtu.be/UhGH8cNG0qs |
| Uploaded | 2026-09-20 |
| Duration | 11:28 (687 s) |
| Captions | YouTube auto-generated (English), 261 segments |
| Fetched | 2026-09-20, via the Hermes `youtube-content` skill (`fetch_transcript.py --timestamps`, `youtube-transcript-api`) |
| Raw fetch | `jev-rag-reranking-prompt-engineering-2026-09-20.transcript.json` — sha256 `f092b8a4dde4693dc6d8f55b1affd52dcf4479e3482e975a6f1ff9309bba78c8` |

## Why it is in this repository

This is a **use-perspective** account of the same specimen E-MT-1/E-MT-2 study
from the inside: not what Jev is, but where someone puts it in a working system
they already have. Its shape is decision points inside an existing pipeline —
"rerank, filter, cite" — with the criteria supplied as the steerable policy
rather than the model being retrained or re-prompted per case. That is the same
posture this programme takes toward Jev (typed judgement as instrument, code
owning the workflow), stated from the outside.

Read it as one practitioner's report, not as evidence.

## Description links (verbatim)

```text
Blogpost: https://typesafe.ai/blog/introducing-system-one-models-and-jev
Colab: https://colab.research.google.com/drive/1bn0CwklTOXAgJhg77iO14G87ouvhr3dE?usp=sharing
```

The Colab is the runnable substance of the video (Python SDK, the reranking
comparison). Captured here as pointers only — nothing from those pages is
fetched or asserted in this file.

## Chapter markers (from the description, verbatim)

```text
00:00 - JEV + RAG
01:15 - Why Vector Search & Cosine Similarity Fail
04:48 - How Steerable Reranking Works
06:35 - JEV vs. LLM Rerankers & Cross-Encoders
08:20 - Architecture Breakdown & Policy Alignment
10:05 - Performance Benchmarks & Key Takeaways
11:00 - Next Steps for Production RAG
```

## Claims as the presenter states them

Quoted numbers and their timestamps, so a reader can go straight to the source
line. All are the presenter's, from a single unreviewed run of his own notebook.

- **Three primitives** (1:02–1:31): binary / true-or-false decision, choice
  ranker over defined criteria, and a score ladder — i.e. the vendored skill's
  Noul / Choice / Score, described from use.
- **The motivating failure** (1:44–2:16): when an organisation's *policy* changes
  but the underlying business logic does not, cross-encoders fail because a
  fixed question–passage pair cannot be steered; LLM rerankers can be steered but
  are "prohibitively expensive". Jev is claimed to give "a really great
  combination of steerability and extreme low cost".
- **A second use shape** (2:17–2:31): validating entities extracted in a graph-RAG
  build — flagged as a future video, not demonstrated here.
- **Concurrency** (6:59–7:38): running questions serially gives ~"17 different
  decisions per second" and took about 40 s; using the offered 64 concurrent
  pools, the same decisions completed in 7.6 s.
- **Criteria are the lever** (7:52–8:41): two criteria sets over the same
  passages — "authoritative version 2" versus "fast workaround" — produce
  materially different rankings. Steering the reranker, not retraining it.
- **Cost against chunk size** (9:21–10:00): Jev versus "Gemini flashlight" and
  "Gemini flesh" (see identifier decoding below). At small chunk sizes cost is
  claimed to be close to the small Gemini model; the gap widens as chunk size
  grows, because Jev's cost is claimed to stay roughly linear in chunk size
  while the LLM reranker's climbs.
- **Reranking accuracy** (10:03–10:38): BM25 full-text baseline top-1 accuracy
  21%; adding Jev as reranker raises it to 54% top-1. The presenter notes the
  final answer's accuracy remains bounded by retrieval quality, and that top-K
  over 10–20 candidate chunks would look better than top-1.

## Reading artefacts of the auto-captions

The transcript body below is **unedited ASR**, so these tables are the decode.
The right column is inference from the video's own context, not a correction
applied to the text.

Model and primitive names misheard:

| In the transcript | Read as |
|---|---|
| "Jeff", "Jude", "Juul", "jewel" | Jev |
| "null", "the null criteria", "our null criteria" | Noul — the primitive, not the value |
| "Two or false classifier", "two or false" | true-or-false classifier |
| "score ladder we" | score ladder (the Score primitive) |
| "Gemini flashlight" | Gemini Flash-Lite (inferred) |
| "Gemini flesh" | Gemini Flash (inferred) |
| "BM5 full text search" | BM25 full-text search |

Pipeline vocabulary misheard:

| In the transcript | Read as |
|---|---|
| "true augmented generation", "retrieval of augmented generation" | retrieval-augmented generation |
| "reancher", "reanker" | reranker |
| "sterility" (at 2:11) | steerability |
| "in steerable instructions" | the steerable instructions |
| "top one accuracy" | top-1 accuracy |
| "there's a two documented generation system" (2:47) | unclear; from 2:49–3:05, a two-document corpus with two competing policy versions the system must resolve between |

## Transcript (timestamped, unedited)

```text
0:00 Jev, it seems to be the only thing
0:02 people are talking about right now, but
0:04 I wanted to create a video showing you
0:06 practical examples of where exactly you
0:09 can use this because there is not much
0:13 content on it to be honest. So I decided
0:16 to combine it with one of my favorite
0:18 topics which is retrieval augmented
0:20 generation.
0:22 Now it's extremely boring but if you
0:24 look at different steps in a true
0:28 augmented generation pipeline you see
0:30 that there are multiple places where you
0:32 have to make a decision such as reancher
0:36 filter or citation and jev turns out to
0:39 be a perfect candidate for those
0:42 decision points. So in this video we're
0:44 going to look at how you can combine Jev
0:48 with the retrieval of augmented
0:49 generation for making these decisions
0:52 and in the process I want to show you
0:55 how to actually think about combining it
0:57 this into your own workflows beyond the
1:00 demos that you have been seeing on the
1:02 internet. Now Jeff basically comes with
1:05 three different primitives that you can
1:07 use. One is binary decision making. You
1:10 can create two or false classifier. The
1:14 second one is choice ranker. So you can
1:16 have multiple different choices, define
1:18 a criteria and based on that select a
1:21 specific choice and then you have a list
1:25 or basically a score ladder we where you
1:28 assign different scores to different
1:31 options. Now for this video we're going
1:33 to be mainly focusing on the reranking
1:37 which I think is a very interesting
1:38 example which actually shows you
1:41 different capabilities of Jev. One very
1:44 interesting topic that I have been
1:45 looking at is what if the policy of an
1:48 organization changes but the actual
1:52 context or the business logic remains
1:55 the same. This is where most of the
1:58 cross encoders actually fail. Now you
2:01 can use something like an LLM reanker
2:05 but it's prohibitively expensive and I
2:09 found that Jev actually gives you a
2:11 really great combination of sterility
2:13 and extreme low cost. Now there are
2:17 other examples like if you are building
2:20 something like graph rag where you need
2:23 to extract knowledge from a corpus you
2:26 can actually use jev to validate
2:28 entities that are extracted. I'll
2:31 probably cover this in another video.
2:34 Now if you're new to Jev and want to
2:36 understand what exactly it does beyond
2:39 the hype I highly recommend to watch
2:41 this video. Okay. So link to the
2:43 notebook is going to be in the video
2:45 description. Now the example we're going
2:47 to be trying to solve here is that
2:49 there's a two documented generation
2:51 system as opposed to extract chunks or
2:55 documents but there could be multiple
2:58 different competing policies different
3:01 versions and the system needs to resolve
3:05 around them. When you run this block
3:07 it's going to ask you for the API key.
3:09 It's dirt cheap. You actually get $5 of
3:12 free credit when you sign up. Here I'm
3:14 using the Python official SDK. You can
3:17 also use the rest API if you like that.
3:20 Then we have a number of different
3:21 helper functions. You have query the
3:23 retrieve documents and then the LLM
3:27 generates a response based on those
3:28 documents. However, in the middle there
3:31 could be some policy changes, updates to
3:35 documentation
3:36 which may not be actually reflected
3:39 in how the answers are being generated
3:43 because your vector distances can only
3:45 measure semantic proximity but not
3:48 correctness under your business logic.
3:51 In a couple of examples I as I mentioned
3:53 there might be changes to the official
3:55 API docs or you might be looking for
3:58 in-depth architectural explanation
4:00 versus quick copy paste snippets. Okay.
4:03 So in the rest of the video the
4:05 benchmark that we're going to be using.
4:06 So there is um one query with multiple
4:09 different candidate points and then
4:12 there are two different official uh
4:14 versions of the policy which are going
4:16 to change over time and we're going to
4:19 test multiple different options. One is
4:21 the cross encoder then LLM reanker and
4:24 then
4:25 Jev with the null criteria and in the
4:29 process you're going to actually see the
4:31 effectiveness of a classifier like Jev
4:34 which has the ability to follow
4:37 instructions. So for this example, our
4:39 query is going to be how do I fix 401
4:42 unauthorized error from the APIs. And to
4:46 make it simple, I have a bunch of
4:47 different chunks that I'm assuming are
4:49 going to be returned from the either
4:53 vector search or full text search along
4:57 with a bunch of metadata. So this is
4:59 what you would expect from a simple
5:01 retrieval augmented generation pipeline.
5:03 Now we want to create a reranker which
5:05 is going to basically rank them and not
5:08 only based on the user query that is
5:10 provided but also additional
5:13 instructions okay but first we need to
5:15 understand primitives specifically how
5:18 do we use criteria as a executable
5:21 policy now as I said there are three
5:23 different primitives we're going to be
5:24 using null which basically assigns
5:27 calibrated probabilities and there are
5:30 two potential outputs whether it's true
5:33 or false criteria. Now in this case
5:36 we're going to be using criteria as a
5:37 policy which instead of prompt in
5:40 instructions the criteria provides
5:42 strict decision boundaries. So here is a
5:46 set of instruction which says as the
5:48 passage directly answer the question.
5:50 Now there are two different criterias.
5:53 So one is the specific answer the
5:55 question asks for and false is share
5:58 words or topics but does not answer it.
6:01 So it's not just true false. You can
6:03 actually find the specific criteria you
6:07 want. And just to show you how it works.
6:10 So here I have a dictionary with
6:12 multiple uh different keys. So for
6:14 example, a direct answer would look
6:15 something like this. Refunds are issued
6:17 to the original payment method within 14
6:20 days. Right? And then there could be
6:22 another example which overlaps with the
6:25 uh topic but doesn't really answer it.
6:27 There might be another one where we have
6:29 keyword based matches, right? And then
6:32 something which is completely off and
6:34 unrelated. Now if we run all of those
6:37 four queries through our null criteria,
6:41 Jude actually assigns the highest
6:43 probability to do the direct answer. And
6:45 for the other the ones, you can actually
6:46 see that the confidence is really low.
6:49 Right? So this can be used as a
6:51 classifier. you can define a threshold
6:54 uh on which you accept the answer below
6:56 which you discard it. Okay. So before
6:59 showing you my experiments I actually
7:01 ran some ideas to see whether we can run
7:05 multiple concurrent questions through
7:07 this system and it seems like it's
7:11 actually great at concurrency. So for
7:13 example, if you were to run multiple
7:16 questions serially,
7:17 in this case we are looking at 17
7:20 different decisions per second, you
7:22 would get about 40 seconds if you run
7:26 those sequentially. However, uh if you
7:29 just use their 64 pools that it offers,
7:33 you can get all of the decisions within
7:35 7.6 seconds, which is incredible. Okay,
7:38 in the rest of the video we're going to
7:40 use Juul as a reanker and I want to
7:43 directly do a comparison with cross
7:45 encoder then LLM as a reanker and jewel
7:48 and see how effective it can be. Now in
7:52 this case we are specifically providing
7:54 it a criteria based on which it needs to
7:57 rank them and there are two different
7:58 criterias. So the first one is let's say
8:01 a policy called authoritative version
8:04 two and we're going to be using the null
8:06 primitive which means there are two
8:08 different options and then there's a
8:10 second one which is fast workaround
8:12 which is not as strict as the
8:15 authoritative one. Now the main takeaway
8:18 is that based on the criteria that you
8:20 define the ranking of the documents that
8:23 you're going to get out of Juul is going
8:27 to be very different. So the criteria or
8:30 the in steerable instructions that you
8:32 provide here can really impact what type
8:35 of ranking you get from the documents
8:38 and this is the beauty because you can
8:41 actually steer your reanker with this.
8:45 Now traditionally we use cross encoders
8:47 for reanking where you provide pair of
8:49 question and the document or chunk right
8:53 but it doesn't really have the ability
8:55 to follow any instructions. So it's kind
8:58 of frozen in time. Okay. But you might
9:00 be thinking there's this second option
9:02 where you can use LLM as a reanker which
9:05 can actually follow instructions. You
9:07 can tell it exactly what the criteria
9:09 should be for ranking and that's
9:11 actually true. However, this is going to
9:14 cost you a lot and it's usually a lot
9:18 slower. Now to show you an example, I
9:21 ran the same prompt and query through
9:23 Jev, Gemini flashlight and Gemini flesh.
9:27 The cost is extremely high compared and
9:31 the most interesting thing is that the
9:33 cost actually depends on how big the
9:35 chunk size is. So for relatively small
9:38 chunk size, both the uh feed and cost of
9:41 Jev are very close to flashlight.
9:44 However, as you increase the uh size,
9:47 you start seeing
9:49 cost pretty much remain like linearly
9:53 incre. However, the biggest difference
9:55 is when you start increasing the chunk
9:57 size. This is where it actually shines.
10:00 Okay. Now, one more thing um the
10:03 accuracy of the final response is really
10:05 dependent on the accuracy of retrieval.
10:07 So, in this case, I created a baseline
10:09 which uses BM5 full text search. Um a
10:12 set of queries that you can actually see
10:14 the baseline accuracy is only 21%. But
10:17 if you add Jev as a reranker, it pushes
10:21 it up to 54%. And this is the top one
10:26 accuracy we're talking about. Right? So
10:28 usually you are going to have uh say 10
10:32 or 20 different chunks and you compute
10:34 top K in which case this is going to be
10:37 a lot more higher. Okay. So this was a
10:40 quick example of how you can use a model
10:42 like Jeff uh to replace different
10:44 decision points in your existing
10:47 workflows. Rag is a really good example
10:49 of that and you can directly plug in
10:53 Jeff as a drop-in replacement in your
10:56 one workflows especially if you have to
10:59 make decision at specific points. They
11:02 don't have to be Je's binary decision.
11:05 Um I'm thinking of creating more content
11:07 on show you you how you can use Jev as a
11:12 a part of your existing workflows. If
11:14 there is any specific topic you're
11:16 interested in, do let me know. I'll be
11:18 happy to cover that in the upcoming
11:20 videos. Anyways, I hope you found this
11:22 video useful. Thanks for watching and as
11:24 always, see you in the next one.
```
