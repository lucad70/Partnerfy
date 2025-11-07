# UI Improvements Prompt for Partnerfy App

Apply the following UI improvements to make the Partnerfy app prettier, matching the reference landing page design:

## Color Scheme & Design System

1. **Update CSS Variables** in `assets/styling/main.css`:
   - Add CSS custom properties for a modern design system:
     - `--accent: #2DD4BF` (teal accent color)
     - `--accent-foreground: #ffffff`
     - `--background: #ffffff`
     - `--foreground: #0f172a` (dark slate)
     - `--muted: #f1f5f9` (light gray)
     - `--muted-foreground: #64748b` (medium gray)
     - `--border: #e2e8f0` (light border)
     - `--input: #e2e8f0`
     - `--ring: #2DD4BF` (focus ring)
     - `--radius: 0.5rem` (border radius)

2. **Typography**:
   - Import Inter font from Google Fonts: `@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');`
   - Update body font-family to: `'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', sans-serif`
   - Remove body margin, add proper line-height and font smoothing
   - Add typography styles for h1, h2, h3 with proper letter-spacing and line-heights

## Component Styling Updates

3. **Panel Sections** (`assets/styling/main.css`):
   - Update `.panel-section` to use CSS variables
   - Add subtle box-shadow: `0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px -1px rgba(0, 0, 0, 0.1)`
   - Add hover effect with enhanced shadow
   - Use `var(--border)` instead of hardcoded colors
   - Update border-radius to use `var(--radius)`

4. **Input Fields**:
   - Update padding to `10px 14px`
   - Use `var(--input)` for border color
   - Add focus state with teal ring: `box-shadow: 0 0 0 3px rgba(45, 212, 191, 0.1)`
   - Border-radius: `calc(var(--radius) - 2px)`

5. **Buttons** (`.button` class):
   - Background: `var(--accent)` (teal)
   - Text color: `var(--accent-foreground)` (white)
   - Remove border, use border-radius: `calc(var(--radius) + 4px)`
   - Add box-shadow: `0 1px 2px 0 rgba(0, 0, 0, 0.05)`
   - Hover: darker teal `#14b8a6` with enhanced shadow and slight transform
   - Add `.button.outline` variant: transparent background, border, hover with muted background

6. **Status Messages**:
   - Use `var(--muted)` background
   - Use `var(--border)` for border
   - Update font-family to modern monospace stack
   - Use CSS variables for colors

7. **Voucher Items**:
   - Update to use CSS variables
   - Hover: `var(--muted)` background with teal border
   - Selected state: teal accent background with ring effect
   - Use `var(--muted-foreground)` for secondary text

8. **Info Boxes**:
   - Info variant: teal accent background `rgba(45, 212, 191, 0.1)` with teal border
   - Warning variant: keep existing yellow styling
   - Use CSS variables for text colors

9. **Loading Indicator**:
   - Add `.loading` class with animated dots
   - Use `var(--muted-foreground)` for color
   - Add CSS animation for dots

10. **Navigation Link**:
    - Add `.nav-link` class for navigation items
    - Use `var(--muted-foreground)` for default color
    - Hover: `var(--foreground)` text with `var(--muted)` background
    - Smooth transitions

## Landing Page Updates (`src/views/landing.rs`)

11. **Header**:
    - Update logo background to `var(--accent)` (teal)
    - Use CSS variables for all colors
    - Border: `1px solid var(--border)` instead of `2px solid #00090C`
    - Navigation link uses `.nav-link` class (no inline event handlers)

12. **Hero Section**:
    - Badge: teal accent background `rgba(45, 212, 191, 0.1)` with teal text
    - Heading: larger font size `3.5rem`, use `var(--foreground)`
    - Description: use `var(--muted-foreground)`
    - Buttons: primary uses `.button`, secondary uses `.button outline`

13. **Features Section**:
    - Icon containers: teal accent background `rgba(45, 212, 191, 0.1)`
    - Border: `1px solid var(--border)` instead of `2px solid #00090C`
    - Use CSS variables for all text colors

14. **How It Works Section**:
    - Numbered badges: teal accent background `var(--accent)` with white text
    - Use CSS variables for all colors
    - Border: `1px solid var(--border)`

15. **Footer**:
    - Logo: teal accent background
    - Border: `1px solid var(--border)`
    - Use CSS variables for colors

## Navbar Updates (`assets/styling/navbar.css`)

16. **Navbar Styling**:
    - Background: `var(--background)` (white) instead of dark
    - Border-bottom: `1px solid var(--border)`
    - Remove negative margins
    - Links: use `var(--muted-foreground)` with hover to `var(--foreground)`
    - First link (Home): `var(--foreground)` with hover to `var(--accent)`
    - Add proper spacing with gap

## Panel Views Updates

17. **Update all panel views** (`src/views/promoter.rs`, `participant.rs`, `partner.rs`):
    - Replace inline loading text with `<div class="loading">Loading</div>`
    - All styling will automatically use the updated CSS classes

## Key Principles

- Use CSS variables throughout for consistency
- Teal accent color (`#2DD4BF`) for primary actions and highlights
- Subtle shadows and hover effects for depth
- Modern spacing and typography
- Smooth transitions on interactive elements
- No inline JavaScript event handlers (use CSS classes instead)

## Files to Modify

1. `assets/styling/main.css` - Main styling with CSS variables and component updates
2. `assets/styling/navbar.css` - Navbar styling updates
3. `src/views/landing.rs` - Landing page component updates
4. `src/views/promoter.rs` - Update loading indicator
5. `src/views/participant.rs` - Update loading indicator
6. `src/views/partner.rs` - Update loading indicator

## Important Notes

- Do NOT use `onmouseenter` or `onmouseleave` in Dioxus - use CSS classes with `:hover` instead
- All colors should use CSS variables for easy theming
- Maintain accessibility with proper contrast ratios
- Keep responsive design considerations

