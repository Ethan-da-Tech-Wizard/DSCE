# ==============================================================================
# Program: gameboy_app.py
# Programmer: Ethan-da-Tech-Wizard
# Last Modified: November 18, 2026
# Purpose: Synthesized Tkinter hybrid application demonstrating DSCE code generation.
#          Combines an arcade paddle/ball bounce game with an interactive arithmetic
#          calculator within a unified retro console UI layout.
# ==============================================================================

# assembled by the DSCE polyglot assembler (Python)
# Standard-library Tkinter only; the whole app is one script with module-level
# state, so the game loop and key handlers share globals.
import tkinter as tk
import tkinter.messagebox as messagebox

# --- CONFIGURATION & PARAMETERS ---
# Tunable parameters, kept at the top so generated variants can change them in one place.
BALL_COLOR = 'red'
BALL_SPEED = 5
# Calculator results above this value are replaced by an "OVER LIMIT!" message.
CALC_LIMIT = 100

# --- APP SETUP ---
# Single main window; geometry is sized to stack the game above the calculator.
root = tk.Tk()
root.title('GameBoy Hybrid Console')
root.geometry('450x650')
root.configure(bg='#2c2c2c')

# ==========================================
# PART 1: PLAYABLE PADDLE GAME
# ==========================================
game_frame = tk.LabelFrame(root, text='Arcade: Bounce Game', bg='#2c2c2c', fg='white', font=('Arial', 12, 'bold'))
game_frame.pack(pady=10, padx=10, fill='both', expand=True)

canvas = tk.Canvas(game_frame, width=400, height=250, bg='black', highlightthickness=0)
canvas.pack(pady=5)

# Canvas items: a 20x20 ball near the top centre and an 80x10 paddle at the bottom (canvas is 400x250).
ball = canvas.create_oval(190, 40, 210, 60, fill=BALL_COLOR)
paddle = canvas.create_rectangle(160, 230, 240, 240, fill='white')
score = 0
score_label = tk.Label(game_frame, text='Score: 0', bg='#2c2c2c', fg='white', font=('Arial', 10, 'bold'))
score_label.pack()

ball_x_speed = BALL_SPEED
ball_y_speed = BALL_SPEED
# Pixels the paddle moves per key press.
paddle_speed = 20

# Key handler for <Left>: move the paddle and clamp it to the left wall.
def move_left(event):
    canvas.move(paddle, -paddle_speed, 0)
    pos = canvas.coords(paddle)
    # Snap back so the paddle (80 wide) stays fully on the canvas.
    if pos[0] < 0:
        canvas.coords(paddle, 0, 230, 80, 240)

# Key handler for <Right>: move the paddle and clamp it to the right wall (x=400).
def move_right(event):
    canvas.move(paddle, paddle_speed, 0)
    pos = canvas.coords(paddle)
    if pos[2] > 400:
        canvas.coords(paddle, 320, 230, 400, 240)

root.bind('<Left>', move_left)
root.bind('<Right>', move_right)

# One animation frame: move the ball, bounce off walls, paddle and floor, then reschedule.
def game_loop():
    # These are module-level state that the function reassigns, hence global.
    global ball_x_speed, ball_y_speed, score
    canvas.move(ball, ball_x_speed, ball_y_speed)
    ball_pos = canvas.coords(ball)
    paddle_pos = canvas.coords(paddle)

    # Side walls reverse horizontal velocity; the ceiling (below) reverses vertical velocity.
    if ball_pos[0] <= 0 or ball_pos[2] >= 400:
        ball_x_speed = -ball_x_speed
    if ball_pos[1] <= 0:
        ball_y_speed = -ball_y_speed

    # Paddle hit: ball bottom reached paddle top and ball is horizontally over the paddle (10 px tolerance). abs() forces the ball upward even if it was already moving up.
    if (ball_pos[3] >= paddle_pos[1] and
        ball_pos[0] >= paddle_pos[0] - 10 and
        ball_pos[2] <= paddle_pos[2] + 10):
        ball_y_speed = -abs(ball_y_speed)
        score += 1
        score_label.config(text=f'Score: {score}')

    # Ball passed the bottom edge (canvas height 250): reset position and score.
    if ball_pos[3] >= 250:
        canvas.coords(ball, 190, 40, 210, 60)
        ball_y_speed = -abs(ball_y_speed)
        score = 0
        score_label.config(text='Game Over! Score Reset to 0')

    # Tk's timer re-invokes this in 30 ms (~33 FPS); using after() keeps the UI responsive instead of a blocking loop.
    root.after(30, game_loop)

# ==========================================
# PART 2: INTERACTIVE CALCULATOR
# ==========================================
calc_frame = tk.LabelFrame(root, text='Calculator Console', bg='#2c2c2c', fg='white', font=('Arial', 12, 'bold'))
calc_frame.pack(pady=10, padx=10, fill='both', expand=True)

display = tk.Entry(calc_frame, font=('Arial', 16), justify='right', bd=5, bg='#1e1e1e', fg='white', insertbackground='white')
display.pack(fill='x', padx=10, pady=5)

# Calculator button handler: '=' evaluates the display, 'C' clears, anything else is appended.
def press_key(key):
    if key == '=':
        try:
            # SECURITY NOTE: eval() runs whatever text is in the display. The Entry is also
            # typeable, so arbitrary Python can be executed; fine for a toy demo only.
            val = eval(display.get())
            if float(val) > float(CALC_LIMIT):
                display.delete(0, tk.END)
                display.insert(tk.END, 'OVER LIMIT!')
            else:
                display.delete(0, tk.END)
                display.insert(tk.END, str(val))
        # Any bad expression (syntax error, divide by zero, ...) shows ERROR.
        except Exception:
            display.delete(0, tk.END)
            display.insert(tk.END, 'ERROR')
    elif key == 'C':
        display.delete(0, tk.END)
    else:
        display.insert(tk.END, key)

# Keypad rows; the labels double as the characters inserted into the expression.
buttons_layout = [
    ['7', '8', '9', '/'],
    ['4', '5', '6', '*'],
    ['1', '2', '3', '-'],
    ['C', '0', '=', '+']
]

grid_frame = tk.Frame(calc_frame, bg='#2c2c2c')
grid_frame.pack()

for r_idx, row in enumerate(buttons_layout):
    for c_idx, val in enumerate(row):
        btn = tk.Button(grid_frame, text=val, font=('Arial', 12, 'bold'), width=5, height=2,
                        bg='#4a4a4a', fg='white', activebackground='#6a6a6a', activeforeground='white',
                        # v=val binds the current label at definition time; without it every button would use the last value of the loop variable.
                        command=lambda v=val: press_key(v))
        btn.grid(row=r_idx, column=c_idx, padx=5, pady=5)

# Start the animation, then hand control to Tk's event loop (blocks until the window closes).
game_loop()
root.mainloop()
