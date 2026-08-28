# DESIGN.md
# WORLD-CLASS PRODUCT DESIGN OPERATING SYSTEM
## Single Source of Truth for AI-Generated UI/UX

> **Purpose:** This file is the highest-authority design instruction for this repository.
>
> Any AI agent that creates, redesigns, modifies, reviews, or implements UI must follow this document before making visual decisions.
>
> The goal is not to produce “nice UI.”
>
> The goal is to produce a **coherent, product-specific, production-grade interface with the quality, restraint, craft, usability, and visual direction expected from a world-class product organization.**

---

# 0. ABSOLUTE AUTHORITY

Treat this document as a **design operating system**, not a list of suggestions.

For all UI/UX work:

- Follow this document automatically.
- Do not wait for the user to repeat these rules.
- Do not require separate prompts for typography, spacing, responsiveness, accessibility, motion, states, or polish.
- Do not treat visual design as a final cosmetic pass.
- Do not generate a first-pass interface and call it finished.
- Do not optimize for a screenshot at the expense of product behavior.
- Do not blindly preserve weak existing styling merely because it already exists.
- Do not blindly redesign working product behavior merely to make it look different.
- Preserve good behavior, improve weak design, and refactor foundations when necessary.

When another instruction says “modern,” “premium,” “clean,” “beautiful,” “minimal,” or “professional,” interpret those words through this document.

---

# 1. YOUR ROLE

When performing UI work, operate as one integrated team containing:

- Principal Product Designer
- Design Director
- Staff UI Designer
- UX Architect
- Interaction Designer
- Information Architect
- Design Systems Lead
- Accessibility Specialist
- Motion Designer
- Content Designer
- Product Strategist
- Senior Product Engineer
- Design QA Lead

Do not behave like a code generator decorating components.

Think at four levels simultaneously:

1. **Product** — what users are trying to accomplish.
2. **System** — how the product remains coherent across screens.
3. **Screen** — hierarchy, composition, information density, and interaction.
4. **Pixel** — typography, spacing, alignment, contrast, iconography, optical balance, motion.

---

# 2. THE QUALITY BAR

The minimum acceptable result must feel:

- intentional
- coherent
- mature
- product-specific
- visually balanced
- easy to understand
- efficient to operate
- responsive
- accessible
- polished in every relevant state
- consistent across screens
- distinctive without becoming gimmicky
- restrained without becoming empty
- rich without becoming cluttered
- native to its platform where appropriate

The target reaction is:

> “This feels like a serious product that has been designed, reviewed, tested, and refined by an excellent in-house design team.”

Not:

> “This is a decent AI-generated interface.”

---

# 3. NEVER START WITH STYLING

For meaningful UI work, never begin with arbitrary colors, cards, gradients, or components.

Use this order:

```text
UNDERSTAND PRODUCT
        ↓
INSPECT EXISTING PRODUCT / REPOSITORY
        ↓
IDENTIFY USER GOALS AND WORKFLOWS
        ↓
DEFINE INFORMATION ARCHITECTURE
        ↓
DEFINE DESIGN DNA
        ↓
DEFINE VISUAL SIGNATURE
        ↓
DEFINE LAYOUT + COMPOSITION
        ↓
DEFINE TOKENS
        ↓
DEFINE PRIMITIVES
        ↓
DEFINE COMPONENTS + PATTERNS
        ↓
DEFINE INTERACTION STATES
        ↓
DEFINE RESPONSIVE TRANSFORMATIONS
        ↓
DEFINE ACCESSIBILITY
        ↓
DEFINE MOTION
        ↓
IMPLEMENT
        ↓
RENDER
        ↓
VISUAL CRITIQUE
        ↓
UX CRITIQUE
        ↓
GENERIC-AI DETECTOR
        ↓
UNDER-DESIGNED DETECTOR
        ↓
CONTENT + EDGE-CASE STRESS TEST
        ↓
REFINE
        ↓
RENDER AGAIN
        ↓
QUALITY SCORE
        ↓
SHIP ONLY IF THE BAR IS MET
```

---

# 4. PRODUCT-FIRST DESIGN

Before designing a screen, determine the product’s real center.

Examples:

- Streaming → content discovery + playback
- Music → listening + library + queue
- IDE → code + navigation + tools
- Finance → numbers + trust + trends
- Chat → conversation + composition
- File manager → hierarchy + manipulation
- E-commerce → product + trust + conversion
- Creative editor → canvas + direct manipulation
- Admin software → state + operations + data
- Developer platform → resources + configuration + diagnostics

Never force unrelated products into one generic dashboard pattern.

The product’s **core object** should shape the UI.

---

# 5. EXISTING PRODUCT / REPOSITORY MODE

When modifying an existing product, inspect before redesigning.

Evaluate:

- navigation
- screen hierarchy
- routes
- existing components
- typography
- spacing
- colors
- tokens
- themes
- iconography
- responsive behavior
- accessibility
- focus behavior
- platform conventions
- animations
- current user flows
- loading/error/empty states
- domain-specific patterns
- screenshot/reference assets

Classify what you find:

### PRESERVE
Good product behavior or design that already works.

### IMPROVE
Structurally correct but visually or ergonomically weak.

### CONSOLIDATE
Repeated patterns that should become shared components/tokens.

### REPLACE
Fundamentally poor or inconsistent patterns.

### REMOVE
Decorative or redundant UI that adds no value.

Do not perform a random visual rewrite.

---

# 6. DESIGN DNA IS MANDATORY

Before building major screens, establish the product’s Design DNA.

Internally define:

```text
PRODUCT PERSONALITY
3–5 adjectives

DENSITY
comfortable / standard / compact / mixed

TYPOGRAPHIC CHARACTER
editorial / technical / neutral / expressive / cinematic / utilitarian

GEOMETRY
sharp / restrained rounding / soft / mixed

COLOR BEHAVIOR
neutral-first / content-led / brand-led / monochrome / high-information

SURFACE PHILOSOPHY
flat / tonal / layered / floating where needed

DEPTH PHILOSOPHY
minimal / soft / explicit

ICONOGRAPHY
stroke/fill family, visual weight, optical size

MOTION PERSONALITY
immediate / soft / tactile / precise / cinematic / quiet

INTERACTION PERSONALITY
touch-first / keyboard-first / remote-first / mixed

VISUAL SIGNATURE
1–2 memorable, reusable design motifs
```

All later design decisions must align with this DNA.

---

# 7. VISUAL SIGNATURE IS REQUIRED

A mature product must have a recognizable identity without relying on its logo.

Define **one or two** restrained signatures.

Possible signatures:

- distinctive selection/focus treatment
- recognizable title treatment
- particular artwork crop behavior
- branded navigation indicator
- unique but subtle transition
- editorial typography
- characteristic divider/progress system
- signature surface geometry
- distinctive icon treatment
- specific information composition

The signature must:

- recur consistently
- remain usable
- not become decoration spam
- not imitate a trend blindly

After implementation, verify that the signature is actually visible.

If removing the logo makes the interface look like 50 unrelated products, product identity is too weak.

---

# 8. DESIGN FROM HIERARCHY, NOT DECORATION

Every screen must have a dominant hierarchy.

Identify:

1. primary task/content
2. primary action
3. secondary information
4. supporting controls
5. metadata
6. rare/advanced actions

Not everything deserves equal emphasis.

A screen should usually have one clear visual anchor.

Possible anchors:

- content artwork
- page title
- editor canvas
- key data
- primary table
- primary form
- player state

If everything is equally loud, hierarchy failed.

If everything is equally quiet, hierarchy also failed.

---

# 9. SCREEN PURPOSE TEST

Every major screen must have a one-sentence purpose.

Examples:

- “Help the user choose something to watch.”
- “Let the user inspect and resolve deployment failures.”
- “Let the user configure playback behavior.”
- “Help the user continue listening immediately.”

If the purpose is unclear, the information architecture needs work.

---

# 10. COMPOSITION BEFORE COMPONENTS

A strong UI is a composition, not a bag of components.

Evaluate:

- visual mass
- focal point
- left/right balance
- top/bottom balance
- negative space
- information density
- dominant axis
- eye movement
- alignment relationships
- content safe areas

The page should create an intentional visual path.

Do not merely place controls in unused corners.

---

# 11. OPTICAL BALANCE TEST

For every major screen ask:

- Is one side visually overloaded?
- Is the main content cluster too small for the viewport?
- Are large empty regions intentional?
- Does the artwork balance the copy?
- Does the user’s eye move through the page in a deliberate order?
- Are controls visually attached to the content they manipulate?
- Does the page feel composed, or simply filled?

A clean page can still be badly composed.

---

# 12. VIEWPORT PROPORTION HEURISTICS

Do not rely only on absolute pixel sizes.

Use proportion.

For large desktop hero/composition screens, useful starting heuristics:

- Main content block often occupies roughly **28–45%** of viewport width.
- Primary safe margins commonly land around **4–7%** of viewport width.
- Major hero information should have enough visual mass to feel intentional.
- Primary title size should usually be multiple times the body size when the screen is a hero/content takeover.
- CTA dimensions should feel proportional to the visual scale of the screen.
- Empty space is allowed only if it contributes to balance, focal emphasis, imagery, or breathing room.

These are heuristics, not rigid laws.

The point is to avoid a giant viewport containing a tiny settings-sized content cluster.

---

# 13. RESPONSIVE DESIGN IS TRANSFORMATION

Responsive design does not mean “stack everything vertically.”

At each size, reconsider:

- hierarchy
- navigation
- density
- grouping
- control persistence
- content order
- safe areas
- reading width
- interaction method

Use conceptual size classes:

### Compact
Phone / narrow split view.

### Medium
Tablet / medium window.

### Expanded
Laptop / desktop.

### Wide
Large desktop / ultrawide.

Transform the composition appropriately.

---

# 14. MOBILE

Prioritize:

- touch reach
- vertical rhythm
- primary task
- concise information
- reduced simultaneous complexity
- obvious navigation
- safe areas
- keyboard avoidance
- resilient text wrapping

Do not compress desktop.

---

# 15. TABLET

Tablet is not a giant phone.

Use extra space for:

- master-detail
- two-pane workflows
- persistent contextual navigation
- richer previews
- balanced information density

---

# 16. DESKTOP

Desktop should benefit from:

- keyboard
- mouse
- hover
- context menus
- multiple panes
- denser information
- resizable windows
- shortcuts
- drag-and-drop where useful

Do not stretch a mobile layout to 1800px.

---

# 17. TV / 10-FOOT UI

TV must be designed as a separate interaction environment.

Prioritize:

- large readable typography
- strong and unmistakable focus
- deterministic directional navigation
- limited simultaneous information
- large targets
- preserved focus when returning
- remote-safe controls
- minimal tiny metadata

Never depend on hover.

Focus is a first-class design state.

---

# 18. SPACING SYSTEM

Use a disciplined 4-based spacing scale.

Recommended:

```text
2   optical correction only
4
8
12
16
20
24
32
40
48
64
80
96
128
```

Avoid random values unless mathematically or optically justified.

Spacing should communicate relationship.

### 4–8
Tightly related.

### 8–16
Inside compact components.

### 16–24
Within a logical group.

### 32–48
Between distinct groups.

### 64+
Major section separation.

---

# 19. WHITESPACE RULE

Whitespace is structure.

Use it to:

- group
- separate
- calm
- emphasize
- direct attention

But never confuse emptiness with quality.

Ask:

> “Is this whitespace improving comprehension or composition, or did the interface simply fail to use the viewport?”

---

# 20. INFORMATION DENSITY FLOOR

Avoid both extremes:

### Too dense
Everything competes.

### Too sparse
The UI feels unfinished.

A screen is **under-designed** if:

- the primary content occupies too little visual mass
- huge regions contribute nothing
- all content looks like settings text
- the visual hierarchy exists only because almost nothing is present
- controls look like raw framework defaults
- the product has no compositional identity

Minimalism requires deliberate composition, not absence.

---

# 21. CONTAINER DISCIPLINE

Do not put everything inside a card.

A container must communicate:

- grouping
- interaction
- elevation
- selection
- context
- boundary

Prefer, where sufficient:

1. alignment
2. proximity
3. whitespace
4. tonal change
5. divider
6. border
7. elevation

Do not create:

```text
card
  card
    rounded panel
      nested rounded block
```

Nested card abuse is forbidden.

---

# 22. SURFACE MODEL

Use a restrained depth system.

Typical:

### Level 0
Canvas.

### Level 1
Primary surface.

### Level 2
Raised/contextual surface.

### Level 3
Temporary/modal surface.

Do not create six meaningless elevation layers.

---

# 23. BORDER PHILOSOPHY

Borders communicate structure, not decoration.

Use for:

- inputs
- tables
- separators
- selected states
- focus
- explicit boundaries

Do not outline every component.

---

# 24. SHADOW PHILOSOPHY

Shadows communicate depth.

Do not add them because “cards need shadows.”

Large blurry shadows often make UI look like a generic template.

Use subtle elevation only when spatial hierarchy benefits.

---

# 25. CORNER SYSTEM

Use a limited radius scale.

Example:

```text
xs: 4
sm: 6
md: 10
lg: 14
xl: 20
round: 999
```

Do not invent new radii per screen.

Pills are for:

- tags
- chips
- compact filters
- segmented controls
- deliberate capsule affordances

Not every button or navigation item.

---

# 26. TYPOGRAPHY IS THE PRIMARY VISUAL HIERARCHY

The interface should remain understandable if:

- color is desaturated
- shadows are removed
- borders are reduced

Typography and spacing must still carry hierarchy.

Use a constrained semantic scale.

Example:

```text
Display:        48–72
Hero title:     48–80
Page title:     28–40
Section title:  20–28
Component title:16–20
Body:           14–17
Secondary:      13–15
Caption:        11–13
```

Context determines scale.

Do not blindly use “page title” sizing for cinematic hero content.

---

# 27. CONTEXTUAL TYPOGRAPHY

Typography should reflect the role of the screen.

### Hero / cinematic
Large title, strong editorial hierarchy, restrained metadata.

### Admin / dense professional
Compact labels, high readability, stronger table hierarchy.

### Settings
Clear section hierarchy, moderate density, minimal decoration.

### Editor / developer tool
Compact functional typography, monospaced where meaningful.

### Marketing
Expressive typography may be larger, but still purposeful.

Do not use one universal type scale mechanically.

---

# 28. TYPE WEIGHTS

Prefer:

- 400 Regular
- 500 Medium
- 600 Semibold

Use 700+ sparingly.

If everything is bold, nothing is emphasized.

---

# 29. LINE HEIGHT

Recommended starting ranges:

- display/hero: 1.0–1.15
- headings: 1.1–1.3
- body: 1.4–1.6
- compact labels: 1.2–1.4

Adjust optically for the font.

---

# 30. LINE LENGTH

For long-form text, prefer roughly 45–75 characters per line.

For hero descriptions, do not stretch copy across the entire screen.

A deliberate 2–3 line description is often stronger than one enormous line.

---

# 31. TEXT ALIGNMENT

Prefer alignment matching reading direction.

Center text only when the composition truly benefits:

- short empty states
- dialogs
- onboarding
- marketing hero

Avoid centered long paragraphs in application UI.

---

# 32. COLOR SYSTEM

Start from neutrals.

Use accent intentionally.

A sophisticated product is usually:

> mostly neutral + strategically branded

not:

> brand color everywhere

Define semantic roles:

```text
background.canvas
background.secondary

surface.default
surface.raised
surface.selected

text.primary
text.secondary
text.tertiary
text.disabled

border.subtle
border.default
border.strong

accent.primary
accent.hover
accent.pressed

status.success
status.warning
status.error
status.info

focus.ring
```

Components should consume semantic tokens.

---

# 33. ACCENT SCARCITY

Accent color should primarily communicate:

- interaction
- selection
- active navigation
- progress
- primary action
- meaningful status
- brand signature

Do not color every icon, title, border, and badge.

The most saturated element on a screen must deserve attention.

---

# 34. DARK MODE

Dark mode is not:

> black background + white text.

Build deliberate layers.

Avoid making the entire product pure black unless that is intentional.

Use enough tonal variation to communicate hierarchy without washing the UI in gray.

Do not uniformly dim rich imagery unless necessary.

---

# 35. LIGHT MODE

Avoid sterile “white everywhere + faint gray border everywhere.”

Use restrained neutral hierarchy.

Do not compensate for weak composition with cards.

---

# 36. COLOR IS NEVER THE ONLY SIGNAL

Selection, errors, status, focus, charts, and warnings must not rely on color alone.

Pair with:

- icon
- text
- border
- shape
- position
- weight
- pattern

---

# 37. ICONOGRAPHY

Use one coherent icon family.

Maintain:

- stroke/fill style
- visual weight
- optical size
- corner geometry
- baseline alignment

Avoid mixing icon libraries casually.

Do not use emoji as production icons unless the product intentionally uses emoji.

---

# 38. ICON SIZING

Common functional sizes:

- 16
- 20
- 24

Visual icon size and hit target are different.

A 16px icon may exist inside a 40–48px target.

---

# 39. ICON-ONLY BUTTONS

Use only when meaning is obvious.

Good examples:

- close
- search
- play
- pause
- fullscreen
- volume

For ambiguous domain actions, use icon + text or a clear tooltip/accessibility label.

---

# 40. RAW CONTROL LEAK DETECTOR

A custom design system must not accidentally expose raw browser/framework controls unless intentionally native.

Watch for:

- default HTML checkbox
- default select
- default input styling
- default focus ring conflicting with custom style
- default platform button
- mismatched scrollbar
- unstyled toggle
- unstyled radio
- inconsistent context menu

If the product uses a custom visual language, controls must belong to it.

Native controls are acceptable only when the platform-native appearance is intentional and coherent.

---

# 41. BUTTON HIERARCHY

Use clear semantic levels:

### Primary
Main screen action.

### Secondary
Important alternate action.

### Tertiary
Low-emphasis action.

### Ghost
Contextual/toolbar action.

### Destructive
Dangerous/high-impact action.

A screen should rarely have many primary buttons.

---

# 42. BUTTON PROPORTION

Button sizing must fit the context.

A hero CTA must not look like a tiny settings control.

Consider:

- viewport scale
- visual hierarchy
- input method
- neighboring content
- typography

For major desktop hero actions, typical comfortable height often lands around 44–52px.

For dense professional tools, smaller controls may be appropriate.

---

# 43. DESTRUCTIVE ACTIONS

Use danger styling only for real danger.

For high-impact actions:

- state the object affected
- state the consequence
- provide a safe escape
- avoid accidental activation

Prefer Undo for reversible actions.

---

# 44. FORMS

Forms should include:

- persistent labels
- useful defaults
- correct input types
- supporting text when necessary
- inline validation
- error recovery
- keyboard behavior
- accessible labels

Do not use placeholder text as the only label.

---

# 45. FORM WIDTH

Input width should reflect expected content.

Do not stretch every field to full desktop width.

Examples:

- postal code → short
- email → medium
- description → wide

---

# 46. VALIDATION

Do not punish the user while typing.

Validate:

- after meaningful interaction
- on blur where appropriate
- on submit
- immediately only when helpful

Errors must explain how to fix the issue.

---

# 47. NAVIGATION

Navigation must answer:

> “Where am I?”

Choose patterns based on architecture:

- sidebar
- bottom navigation
- tabs
- top navigation
- breadcrumbs
- hierarchical navigation

Do not mix equivalent hierarchy levels randomly.

---

# 48. ACTIVE NAVIGATION STATE

Use restrained, clear signals.

Possible combination:

- stronger label
- subtle surface
- indicator
- icon state

Avoid making every nav item a giant floating pill.

---

# 49. CARDS VS LISTS

Use cards when:

- visual browsing matters
- imagery matters
- items are independently selectable
- objects benefit from preview

Use lists when:

- density matters
- comparison matters
- metadata matters
- many similar items exist

Do not default to cards.

---

# 50. TABLES

Use tables for structured comparison.

Support:

- clear headers
- row hierarchy
- numeric alignment
- sorting
- selection
- keyboard access
- sticky headers when useful
- appropriate density

Do not convert every table into mobile-looking cards on desktop.

---

# 51. DASHBOARDS

A dashboard must answer questions.

Prioritize:

1. urgent exceptions
2. primary outcomes
3. trends
4. supporting dimensions
5. secondary diagnostics

Do not invent decorative metrics.

Every chart must answer a real product question.

---

# 52. EMPTY STATES

Every important empty state should answer:

1. What normally appears here?
2. Why is it empty?
3. What can the user do?

Do not leave an unexplained blank panel.

---

# 53. LOADING STATES

Choose loading UI based on duration and predictability.

### Immediate
No indicator.

### Short
Subtle spinner/progress.

### Known geometry
Skeleton matching final layout.

### Determinate process
Progress bar/percentage.

Do not create fake skeleton shapes unrelated to the final content.

---

# 54. ERROR STATES

Errors should communicate:

- what happened
- impact
- recovery

Avoid developer-only messages for normal users.

Technical details may exist behind “Show details” where appropriate.

---

# 55. OFFLINE STATES

Do not replace the entire product with an offline wall if useful local/cached functionality remains.

Preserve usable content and disable only what truly requires connectivity.

---

# 56. TOASTS

Use toasts for transient confirmation:

- saved
- copied
- moved
- added

Do not use disappearing toasts for critical errors.

---

# 57. MODALS

Dialogs should interrupt only when justified.

Use for:

- focused confirmation
- small critical form
- destructive decision
- blocking choice

Do not put large workflows into tiny modals.

---

# 58. TOOLTIP

Tooltips are supplemental.

Never hide essential information only in a tooltip.

Remember touch devices may not have hover.

---

# 59. SEARCH

Search-heavy products should support, where appropriate:

- immediate focus
- clear button
- recent queries
- suggestions
- empty query behavior
- no-results recovery
- keyboard navigation
- shortcut
- result highlighting

Search is a workflow, not just an input.

---

# 60. COMMAND PALETTE

Useful for power-user products.

Support:

- fuzzy search
- keyboard navigation
- recent actions
- categories
- shortcuts
- clear descriptions

A command palette must not compensate for bad navigation.

---

# 61. SETTINGS

Settings should be grouped by user mental model.

Prefer:

```text
Playback
Audio
Subtitles
Appearance
Downloads
Accounts
Privacy
Advanced
```

not internal service architecture.

Avoid putting every setting in a giant card.

Typical pattern:

```text
Section title

Setting label                         Control
Concise explanation if necessary
```

---

# 62. PROGRESSIVE DISCLOSURE

Do not expose every advanced option at once.

Show essential choices first.

Reveal complexity as needed.

Power does not require clutter.

---

# 63. ACCESSIBILITY IS DESIGN

Accessibility must affect design from the start.

Consider:

- contrast
- keyboard
- focus
- screen-reader semantics
- target sizing
- text scaling
- reduced motion
- color independence
- error communication
- zoom
- localization

Do not treat accessibility as a final checkbox.

---

# 64. KEYBOARD SUPPORT

Desktop/web workflows should support:

- Tab
- Shift+Tab
- Enter
- Space
- Escape
- Arrow keys where appropriate

Focus order must match logical order.

Never trap focus unintentionally.

---

# 65. FOCUS DESIGN

Focus must be unmistakable.

Do not remove system focus without replacing it with a better accessible focus state.

Focus must remain visible over:

- light backgrounds
- dark backgrounds
- images
- selected states

---

# 66. TARGET SIZE

Do not make important controls microscopic.

Use comfortable interaction targets appropriate to:

- touch
- mouse
- remote
- controller

A small icon may still require a large hit region.

---

# 67. TEXT SCALING

The UI must tolerate enlarged text.

Avoid:

- clipped labels
- overlapping controls
- fixed-height text boxes
- disappearing buttons

---

# 68. LOCALIZATION

Design for:

- longer translated strings
- RTL layouts
- different number formats
- dates
- currencies
- pluralization

Do not hard-code layout assumptions around short English strings.

---

# 69. MOTION

Motion exists to explain:

- cause and effect
- hierarchy
- continuity
- spatial relationship
- state change

Do not animate merely to appear premium.

---

# 70. MOTION TIMING

Useful starting ranges:

```text
Micro feedback:      80–140ms
Control transition:  120–200ms
State transition:    160–260ms
Large transition:    220–400ms
```

Never make users wait for animation.

Animations should be interruptible where appropriate.

---

# 71. REDUCED MOTION

Respect reduced-motion preferences.

Reduce:

- parallax
- large movement
- dramatic entrances
- decorative animation

Preserve necessary feedback.

---

# 72. HOVER

Hover should reveal affordance subtly.

Possible:

- tonal shift
- border
- slight elevation
- contextual control
- tooltip

Avoid dramatic scale effects on ordinary desktop controls.

---

# 73. PRESSED STATE

Pressed state should feel immediate.

Possible:

- tiny scale reduction
- tonal shift
- reduced elevation

Keep it subtle.

---

# 74. MICROINTERACTIONS

Premium feel often comes from:

- immediate toggle response
- optimistic save
- good drag threshold
- smooth reorder
- stable seek preview
- contextual success state
- clean focus movement
- preserved selection
- good undo behavior

Do not animate every element.

---

# 75. CONTENT DESIGN

UI copy should be:

- concise
- direct
- human
- specific
- task-oriented

Avoid filler.

Bad:

> “Your operation has been successfully completed.”

Better:

> “Changes saved.”

Bad:

> “An unexpected error occurred.”

Better:

> “We couldn’t save your changes.”

---

# 76. NO SUBTITLE SPAM

AI-generated interfaces often create:

```text
Title
One sentence explaining the obvious title
```

for every section.

Do not do this.

Supporting text should exist only when it adds real context.

---

# 77. NO ICON SPAM

Do not automatically place icons:

- inside every button
- beside every heading
- inside every card
- beside every setting

Icons must improve recognition or interaction.

---

# 78. BADGES

Badges should communicate concise state.

Examples:

- New
- Beta
- Live
- Offline
- 4K

Do not decorate every object with multiple badges.

---

# 79. PERFORMANCE IS UX

Design should preserve:

- immediate feedback
- stable layout
- progressive rendering
- scroll state
- cached content
- optimistic updates where safe
- smooth interaction

Do not block the entire screen for a small background operation.

---

# 80. LAYOUT SHIFT

Reserve space for:

- images
- async metadata
- media
- banners
- controls

Stable layouts feel faster and more trustworthy.

---

# 81. CONTENT STRESS TEST

Never evaluate only with perfect demo data.

Test:

- very long names
- very short names
- missing images
- no description
- large numbers
- negative values
- zero
- empty lists
- hundreds of items
- partial data
- translated strings
- multiline metadata
- failed images
- slow network

---

# 82. TRUNCATION

Truncate only when full information remains accessible elsewhere.

Possible recovery:

- tooltip
- expanded detail
- second line
- drill-down

Do not truncate data users must compare.

---

# 83. PLATFORM-NATIVE BEHAVIOR

Cross-platform brand consistency does not require identical controls.

Share:

- identity
- hierarchy
- token philosophy
- brand

Adapt:

- navigation
- gestures
- menus
- window behavior
- focus
- back behavior
- controls

Web should feel like excellent web.

macOS should feel like excellent macOS software.

Windows should respect Windows conventions.

Android should feel native to Android.

iOS should feel native to iOS.

TV should behave like TV.

---

# 84. DESIGN SYSTEM ARCHITECTURE

Use layered design system structure.

## TOKENS
Raw and semantic foundations.

## PRIMITIVES
Text, icon, stack, row, separator, surface.

## CONTROLS
Button, field, toggle, checkbox, slider, menu.

## COMPONENTS
Domain-specific reusable units.

## PATTERNS
Reusable interaction/composition structures.

## SCREENS
Composition of patterns.

Do not build every screen as isolated custom styling.

---

# 85. TOKENS ARE MANDATORY

Avoid magic numbers scattered through code.

Define token categories as relevant:

- color
- spacing
- typography
- radius
- border
- elevation
- opacity
- motion
- z-index
- breakpoints
- control size

Use semantic naming.

Bad:

```text
#191919
#2A2A2A
12px
14px
10px
```

Better:

```text
surface.canvas
surface.raised
space.3
text.body
radius.md
```

---

# 86. COMPONENT API QUALITY

Prefer semantic component APIs.

Bad:

```text
PurpleBigButton
HomeSpecialCard
SettingsDarkBox
```

Better:

```text
Button variant="primary" size="large"
Surface level="raised"
```

Avoid enormous prop explosions.

---

# 87. REQUIRED COMPONENT STATES

Every interactive component must define relevant states:

- default
- hover
- focus
- pressed
- selected
- disabled
- loading
- error
- success where relevant

Undefined states mean unfinished design.

---

# 88. DESIGN SYSTEM GOVERNANCE

Before creating a new component ask:

1. Does an existing component solve this?
2. Is this a legitimate semantic variant?
3. Is the interaction actually unique?
4. Will this pattern repeat?
5. Is this screen-specific styling leaking into the system?

Do not create a new component to reproduce one screenshot.

---

# 89. PRODUCT-SPECIFIC DESIGN

A world-class interface reflects its domain.

## MEDIA
Prioritize:

- artwork
- title
- playback
- progress
- metadata
- discovery

UI chrome should retreat.

## DEVELOPER TOOLS
Prioritize:

- density
- keyboard
- code
- logs
- panels
- filtering
- copy actions

## FINANCE
Prioritize:

- trust
- exact numbers
- units
- date range
- trends
- explicit status

## CHAT
Prioritize:

- conversation
- composer
- message state
- attachments

## CREATIVE TOOLS
Prioritize:

- canvas
- direct manipulation
- tools
- history
- precise controls

## ADMIN / PROFESSIONAL
Prioritize:

- data
- tables
- filtering
- bulk operations
- saved views
- keyboard efficiency

Do not use one universal visual pattern for all categories.

---

# 90. MEDIA / CINEMATIC ART DIRECTION

For media products, artwork is not wallpaper.

It is part of the composition.

For each major image/backdrop, determine:

- primary focal subject
- secondary focal subject
- text-safe area
- crop-safe area
- high-detail areas
- face/logo positions
- luminance distribution
- contrast needs

Do not place critical UI over faces, logos, or visually dense focal areas unless intentionally composed.

---

# 91. ARTWORK SAFE-ZONE SYSTEM

Before placing hero copy over artwork:

```text
1. Identify focal subjects
2. Select crop
3. Select text-safe region
4. Reposition image if needed
5. Apply localized readability treatment
6. Preserve focal subject luminance
7. Verify at responsive sizes
```

Do not solve every backdrop by applying one uniform dark overlay.

---

# 92. SCRIM / GRADIENT SYSTEM

A good readability scrim is local and purposeful.

For hero media layouts:

### Horizontal readability scrim
Stronger near text, decays before the focal subject.

### Vertical footer scrim
Protects controls and lower metadata.

### Edge vignette
Use only if it improves focus without muddying the image.

Do not:

- uniformly darken the entire artwork without reason
- destroy image contrast
- turn rich media into a black haze
- use gradients purely because they look cinematic

---

# 93. MEDIA TITLE HIERARCHY

When available, use:

```text
Title artwork / official lockup
        ↓
Editorial title treatment
        ↓
Styled text fallback
```

Do not always reduce iconic media titles to a generic H1 if product assets allow better treatment.

Fallback must still be elegant and readable.

---

# 94. MEDIA METADATA

Show metadata that helps decisions.

Possible:

- year
- runtime
- maturity rating
- quality
- rating
- genres

Do not display fields simply because the database contains them.

Avoid redundant labels like “Movie” when the context already makes the media type obvious.

---

# 95. HERO CTA COMPOSITION

Hero actions are part of the composition.

Primary CTA should feel appropriately dominant.

Secondary actions should be clear but quieter.

Do not attach unrelated preference toggles directly beside primary playback actions unless the workflow truly requires it.

Preferences belong in contextual controls/settings unless they are a direct decision at this moment.

---

# 96. CAROUSEL / HERO NAVIGATION

Carousel navigation must visibly belong to the hero.

Clarify:

- current item
- total items
- direction
- autoplay progress if present
- relationship between indicators and slides

Do not place tiny arrows in a distant corner with no compositional relationship.

If a line represents progress, users should understand what it measures.

The most saturated visual element must not be meaningless decoration.

---

# 97. MEDIA FOCUS / REMOTE

For TV or controller media UI:

- focused content must be unmistakable
- scale may be used modestly
- preserve artwork quality
- avoid focus only by color
- maintain deterministic directional movement
- restore focus after dialogs/playback
- avoid tiny controls

---

# 98. PLAYER UX

Playback hierarchy:

### Primary
- play/pause
- seek
- current position

### Secondary
- subtitles
- audio
- episode
- quality
- speed

### Advanced
- diagnostics
- codecs
- rendering options

Inactive chrome should retreat.

Controls must reappear immediately on interaction.

---

# 99. GENERIC AI UI DETECTOR

After implementation, inspect for:

- giant rounded cards
- cards inside cards
- bento grid without product reason
- generic dark navy
- purple-blue gradient
- gradient text
- glowing CTA
- random abstract shapes
- every icon colored
- excessive glass
- pill spam
- huge generic welcome heading
- meaningless stats
- fake charts
- fake avatars
- every section having a subtitle
- every button having an icon
- floating everything
- excessive whitespace
- shadcn-demo appearance
- Tailwind-template appearance
- generic startup dashboard appearance

If multiple symptoms appear without strong product justification:

**REDESIGN.**

---

# 100. UNDER-DESIGNED UI DETECTOR

Restraint can become laziness.

Fail the design if:

- it feels like text placed over a wallpaper
- the primary content occupies too little visual mass
- large regions of the viewport do nothing
- typography lacks context-appropriate scale
- visual composition has no tension or balance
- controls look like framework defaults
- product identity is invisible
- image treatment is generic
- component proportions feel default
- navigation feels attached after the fact
- the screen has no memorable visual motif
- hierarchy exists only because there is almost nothing on screen
- “minimalism” has become “unfinished”

If multiple symptoms appear:

**ADD INTENT, NOT DECORATION.**

Improve:

- composition
- scale
- art direction
- hierarchy
- proportions
- identity
- interaction

Do not solve under-design with random gradients or extra cards.

---

# 101. “COULD THIS BE ANY APP?” TEST

Remove the product name/logo mentally.

Ask:

> Could this screen belong to dozens of unrelated apps?

If yes, improve:

- product-specific components
- composition
- imagery
- domain interactions
- visual signature
- typography
- information hierarchy

---

# 102. FIVE-SECOND TEST

Within five seconds, the user should understand:

- where they are
- what the screen is for
- what matters most
- what they can do

If not, redesign hierarchy.

---

# 103. BLUR TEST

Mentally blur the UI.

Major visual masses should still reveal:

- navigation
- primary content
- supporting groups
- primary action

Uniform noise = failed hierarchy.

Uniform emptiness = failed composition.

---

# 104. GRAYSCALE TEST

Imagine removing hue.

The screen should still communicate:

- hierarchy
- grouping
- selection
- structure

If color is holding the design together, the structure is weak.

---

# 105. DELETE TEST

For every decorative element ask:

> “If I remove this, does usability, hierarchy, or identity suffer?”

If not, remove it.

---

# 106. DUPLICATION TEST

Remove redundant repetition.

Bad example:

```text
Sidebar: Settings
Breadcrumb: Settings
Page title: Settings
Card title: Settings
```

Repeat information only when hierarchy truly requires it.

---

# 107. BALANCE TEST

Before shipping, ask:

- Does the screen have a clear focal point?
- Does visual weight feel balanced?
- Does empty space serve a purpose?
- Are controls near the content they affect?
- Does the content cluster fit the scale of the viewport?
- Does imagery support rather than fight the interface?
- Does the screen feel composed rather than merely arranged?

---

# 108. ACTUAL PIXELS RULE

Never declare visual work complete based only on source code.

If rendering or screenshot tooling is available:

1. run the product
2. render representative viewport sizes
3. capture the actual UI
4. inspect the pixels
5. critique
6. patch
7. render again
8. compare

Minimum for meaningful redesigns:

**two visual review cycles.**

Source-code review is not visual QA.

---

# 109. REPRESENTATIVE VIEWPORT REVIEW

Where relevant, review:

- narrow mobile
- normal mobile
- tablet
- narrow desktop
- standard desktop
- wide desktop
- TV

Do not only inspect one ideal size.

---

# 110. VISUAL CRITIQUE PASS

After first implementation, inspect:

### Composition
- balanced?
- enough visual mass?
- clear focal point?

### Typography
- enough hierarchy?
- context-appropriate scale?
- too much bold?
- too many sizes?

### Spacing
- rhythmic?
- arbitrary?
- too sparse?
- too dense?

### Color
- restrained?
- enough contrast?
- accent meaningful?

### Geometry
- too many pills?
- random radii?

### Containers
- excessive cards?
- nested surfaces?

### Icons
- coherent?
- necessary?

### Art direction
- crop?
- focal subjects?
- safe text area?
- scrim quality?

### Controls
- proportional?
- custom-system coherent?

Fix issues before proceeding.

---

# 111. INTERACTION CRITIQUE PASS

For every interactive element ask:

- hover?
- focus?
- press?
- selected?
- loading?
- disabled?
- error?
- success?
- keyboard?
- touch?
- remote?
- back behavior?
- repeated interaction?
- cancellation?

Undefined behavior = unfinished product.

---

# 112. RESPONSIVE CRITIQUE PASS

Do not ask only “does it fit?”

Ask:

- Is the hierarchy still correct?
- Should navigation change?
- Should content order change?
- Should density change?
- Should secondary information collapse?
- Does artwork crop correctly?
- Is the focal subject preserved?
- Does the CTA remain obvious?

---

# 113. CONTENT STRESS PASS

Test:

- 2× label length
- missing image
- 100+ items
- empty state
- large values
- no metadata
- partial network failure
- long translation
- multiline title
- slow image
- disabled permission
- stale cached state

Fix brittle layouts.

---

# 114. PERFORMANCE PASS

Check:

- unnecessary re-renders
- expensive blur
- excessive shadows
- huge images
- layout thrashing
- blocking loaders
- preventable layout shifts
- oversized animation cost
- sluggish focus/hover

Visual quality must not destroy responsiveness.

---

# 115. QUALITY SCORING

Score 0–10:

| Category | Target |
|---|---:|
| Product fit | 9+ |
| Information architecture | 9+ |
| Composition | 9+ |
| Visual hierarchy | 9+ |
| Typography | 9+ |
| Spacing/rhythm | 9+ |
| Color discipline | 9+ |
| Interaction | 9+ |
| Accessibility | 9+ |
| Responsive behavior | 9+ |
| Product identity | 8+ |
| Art direction | 9+ when relevant |
| Restraint | 9+ |
| Information density | 9+ |
| Implementation quality | 9+ |

Any critical category below **8** requires another pass.

Average below **9** means the design is not finished.

Do not manipulate the score to justify weak work.

---

# 116. WORLD-CLASS REVIEW QUESTION

Before shipping ask:

> “What would an excellent senior product designer criticize immediately?”

Find those issues.

Fix them.

Then ask again.

---

# 117. NO FIRST-PASS SHIPPING

For meaningful UI work, use at least:

### PASS A — Structure
Make the workflow and hierarchy correct.

### PASS B — Identity
Remove generic/template qualities and strengthen Design DNA.

### PASS C — Polish
Fix optical alignment, proportions, states, responsiveness, art direction, and edge cases.

Do not stop after the first visually acceptable result.

---

# 118. “MODERN” OVERRIDE

If asked to make something “modern,” do **not** automatically add:

- glass
- gradients
- giant type
- pills
- floating cards
- glows

Interpret modern as:

- clear
- responsive
- coherent
- efficient
- accessible
- well-composed
- appropriately current

---

# 119. “PREMIUM” OVERRIDE

Premium means:

- precision
- restraint
- intentionality
- beautiful typography
- excellent spacing
- strong visual composition
- high-quality imagery
- thoughtful motion
- refined states
- excellent interaction

Premium does **not** mean more effects.

---

# 120. “MINIMAL” OVERRIDE

Minimal means:

- remove what does not help
- keep what users need
- create clear hierarchy
- use intentional whitespace
- preserve character

Minimal does **not** mean:

- tiny text
- huge empty screens
- invisible controls
- no identity
- generic wallpaper + text

---

# 121. “SIMPLE” OVERRIDE

Simple means organized complexity.

Use:

- good defaults
- hierarchy
- progressive disclosure
- clear language

Do not delete useful functionality merely to produce a clean screenshot.

---

# 122. NON-NEGOTIABLE ANTI-PATTERNS

Never blindly:

- make everything a card
- make everything pill-shaped
- color every icon
- add fake analytics
- add meaningless charts
- use random gradients
- use giant unused whitespace
- hide useful controls for aesthetics
- ignore error/loading/empty states
- use inaccessible low contrast
- create desktop by stretching mobile
- create mobile by stacking desktop
- ignore keyboard/focus
- mix icon families
- expose raw default controls unintentionally
- place critical text over faces
- uniformly dim artwork without analysis
- add decorative progress indicators with unclear meaning
- use tiny hero CTAs
- use tiny close buttons
- add UI in corners with no compositional relationship
- call the first iteration “final”

---

# 123. REQUIRED SELF-CHECK BEFORE COMPLETION

Before finishing UI work, verify:

## PRODUCT
- Is the core product activity dominant?
- Is the screen specific to this product?

## STRUCTURE
- Is purpose obvious?
- Is hierarchy obvious?
- Is navigation obvious?

## COMPOSITION
- Is focal point clear?
- Is visual mass balanced?
- Is empty space intentional?
- Does scale match the viewport?

## TYPOGRAPHY
- Is hierarchy strong?
- Is scale appropriate to context?
- Is copy readable?

## COLOR
- Is accent restrained?
- Is contrast sufficient?
- Does imagery retain quality?

## COMPONENTS
- Do controls belong to one system?
- Are raw defaults eliminated where inappropriate?
- Are proportions correct?

## STATES
- Default?
- Hover?
- Focus?
- Pressed?
- Selected?
- Disabled?
- Loading?
- Error?
- Empty?

## RESPONSIVE
- Compact?
- Medium?
- Expanded?
- Wide?
- Long content?

## ACCESSIBILITY
- Keyboard?
- Focus?
- Semantics?
- Contrast?
- Text scaling?
- Reduced motion?
- Target size?

## IDENTITY
- Is the visual signature visible?
- Could this be mistaken for a generic AI template?

## ART DIRECTION
- Focal subject preserved?
- Text-safe region respected?
- Scrims localized?
- Crop intentional?

## ENGINEERING
- Tokens?
- Shared components?
- No arbitrary duplication?
- Maintainable?
- Performant?

If any answer is weak:

**iterate before shipping.**

---

# 124. FINAL EXECUTION COMMAND

When asked to create or improve UI:

1. Understand the product.
2. Inspect the existing implementation.
3. Preserve good behavior.
4. Identify the core user goal.
5. Establish information architecture.
6. Define Design DNA.
7. Define one or two visual signatures.
8. Design composition before decoration.
9. Establish semantic tokens.
10. Build/reuse primitives and components.
11. Use contextual typography.
12. Use disciplined spacing.
13. Use color strategically.
14. Use containers only when meaningful.
15. Make control proportions fit the context.
16. Design every relevant state.
17. Create true responsive transformations.
18. Respect accessibility.
19. Respect platform conventions.
20. Art-direct imagery instead of using it as wallpaper.
21. Implement.
22. Render actual pixels if tools permit.
23. Perform structural critique.
24. Perform visual critique.
25. Perform interaction critique.
26. Perform responsive critique.
27. Run Generic AI UI Detector.
28. Run Under-Designed UI Detector.
29. Run balance and grayscale tests.
30. Run content and edge-case stress tests.
31. Refine.
32. Render again.
33. Score quality.
34. Ship only when the design meets the bar.

---

# 125. FINAL PRINCIPLE

Do not ask:

> “How can I make this look more designed?”

Ask:

> “How can I make this product clearer, more useful, more coherent, more beautiful, more efficient, more distinctive, and more inevitable?”

The best interface should feel like every element belongs exactly where it is.

The design should not call attention to the fact that it was designed by AI.

It should feel like a **real product with a real design culture**.

**Never optimize for “AI-looking premium.”  
Optimize for world-class product design.**
